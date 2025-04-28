import os
import subprocess
import ipaddress
import shutil
import re

from .base import TGA

class SixScanTGA(TGA):
    def __init__(self, github_url: str, clone_directory: str = "repos"):
        super().__init__(github_url, clone_directory)

    def initialize(self) -> None:
        # 1) Clone the repo
        self.clone()
        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))

        # 2) Build the C++ tools (scanner, strategy, etc.)
        subprocess.run(["./bootstrap"], cwd=repo_path, check=True)
        subprocess.run(["./configure"], cwd=repo_path, check=True)
        subprocess.run(["make", "-j"], cwd=repo_path, check=True)

        # 3) Prep a Python 3.7 venv with all 6Gen script deps
        #    - numpy, pandas, pyasn, pytricia, radix, iso3166, multiping
        self._initialize_python(
            "3.7.16",
            ["numpy", "pandas", "pyasn", "pytricia", "radix", "iso3166", "multiping"]
        )

    def train(self, ipv6_addresses: list[str]) -> None:
        if not self.env_python:
            raise RuntimeError("Call initialize() first.")

        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        self._output_dir = os.path.join(repo_path, "output", "tga")  # keep for generate()
        os.makedirs(self._output_dir, exist_ok=True)

        # Write exploded seeds to disk
        self._seeds_file = os.path.join(self._output_dir, "seeds.txt")
        with open(self._seeds_file, "w") as f:
            for addr in ipv6_addresses:
                try:
                    exploded = ipaddress.IPv6Address(addr).exploded
                except ipaddress.AddressValueError:
                    raise ValueError(f"Invalid IPv6 address: {addr!r}")
                f.write(exploded + "\n")

        print(f"Wrote {len(ipv6_addresses)} seeds to {self._seeds_file}")

    def generate(self, count: int) -> list[str]:
        if not self.env_python:
            raise RuntimeError("Call initialize() first.")
        if not hasattr(self, "_seeds_file"):
            raise RuntimeError("seeds.txt not found; run train(...) first.")

        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        # Work in a fresh run directory under the toy generator
        script_dir = os.path.join(repo_path, "toyscanner", "6Gen")
        run_dir = os.path.join(script_dir, "run_output")
        shutil.rmtree(run_dir, ignore_errors=True)
        os.makedirs(run_dir, exist_ok=True)

        # Patch the 6gen.py script for this run:
        script_path = os.path.join(script_dir, "6gen.py")
        with open(script_path, "r") as f:
            code = f.read()

        # 1) Use our seed file
        code = re.sub(
            r'input\s*=\s*".*?"',
            f'input = "{self._seeds_file}"',
            code
        )
        # 2) Set the budgetLimit to our count
        code = re.sub(
            r'budgetLimit\s*=\s*\d+',
            f"budgetLimit = {count}",
            code
        )
        # 3) Redirect its hard-coded Sim_experiment_downsampling... paths into our run_dir
        code = code.replace(
            "./Sim_experiment_downsampling3k_9k",
            run_dir.replace("\\", "/")  # normalize on Windows if needed
        )

        # Write out a temporary runner script
        runner = os.path.join(run_dir, "6gen_run.py")
        with open(runner, "w") as f:
            f.write(code)

        # Execute it
        proc = subprocess.run(
            [self.env_python, runner],
            cwd=script_dir,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        if proc.returncode != 0:
            raise RuntimeError(f"6gen.py failed:\n{proc.stderr}")

        # Read back the targets it emitted
        target_file = os.path.join(run_dir, "targets")
        if not os.path.isfile(target_file):
            raise RuntimeError("No targets file created; did 6gen.py run correctly?")

        with open(target_file) as f:
            targets = [line.strip() for line in f if line.strip()]

        print(f"Discovered {len(targets)} addresses")
        return targets
