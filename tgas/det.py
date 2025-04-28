import os
import subprocess
import ipaddress
import shutil

from .base import TGA

class SixDETTGA(TGA):
    def __init__(self, github_url: str, clone_directory: str = "repos"):
        super().__init__(github_url, clone_directory)

    def initialize(self) -> None:
        # Clone and prepare Python 3.7
        self.clone()
        self._initialize_python("3.7.16", [])

    def train(self, ipv6_addresses: list[str]) -> None:
        if not self.env_python:
            raise RuntimeError("Call initialize() first.")

        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        output_dir = os.path.join(repo_path, "output")
        seeds_file = os.path.join(output_dir, "seeds.txt")

        # Make output directory
        os.makedirs(output_dir, exist_ok=True)

        print("Writing seeds...")
        with open(seeds_file, "w") as f:
            for addr in ipv6_addresses:
                try:
                    exploded = ipaddress.IPv6Address(addr).exploded
                except ipaddress.AddressValueError:
                    raise ValueError(f"Invalid IPv6 address: {addr!r}")
                f.write(exploded + "\n")

        print(f"Wrote {len(ipv6_addresses)} seeds to {seeds_file}")

    def generate(self, count: int) -> list[str]:
        if not self.env_python:
            raise RuntimeError("Call initialize() first.")

        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        output_dir = os.path.join(repo_path, "output")
        seeds_file = os.path.join(output_dir, "seeds.txt")

        if not os.path.isfile(seeds_file):
            raise RuntimeError("seeds.txt not found; run train(...) first.")

        # read the first seed as source_ip
        with open(seeds_file) as f:
            first = next((line.strip() for line in f if line.strip()), None)

        if not first:
            raise RuntimeError("seeds.txt is empty")

        source_ip = first

        # Delete old zmap directory
        zmap_dir = os.path.join(output_dir, "zmap")
        if os.path.exists(zmap_dir):
            shutil.rmtree(zmap_dir)
        os.makedirs(zmap_dir)

        cmd = [
            self.env_python,
            "DynamicScan.py",
            "--input", seeds_file,
            "--output", output_dir,
            "--budget", str(count),
            "--IPv6", source_ip,
        ]

        print("Running scan...")
        proc = subprocess.run(
            cmd,
            cwd=repo_path,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )

        if proc.returncode != 0:
            raise RuntimeError(f"DynamicScan.py failed:\n{proc.stderr}")

        discovered = set()
        for fn in os.listdir(zmap_dir):
            if fn.startswith("scan_output_") and fn.endswith(".txt"):
                for line in open(os.path.join(zmap_dir, fn)):
                    addr = line.strip()
                    print(addr)
                    if addr:
                        discovered.add(addr)

        print(f"Discovered {len(discovered)} addresses")

        return list(discovered)
