import os
import subprocess
import ipaddress
import random
import re

from .base import TGA

class SixGraphTGA(TGA):
    def __init__(self, github_url: str, clone_directory: str = "repos"):
        super().__init__(github_url, clone_directory)

    def initialize(self) -> None:
        # 1) Clone the 6Graph repo
        self.clone()
        self._initialize_python("3.12.4", ["numpy", "IPy", "networkx"])

        self._patch_initialize(["PatternMining.py"])

        self._patch_replace(
            "PatternMining.py",
            "return len(arrs) / xi",
            "if xi == 0: return float('inf')\n    return len(arrs) / xi"
        )

    def train(self, ipv6_addresses: list[str]) -> None:
        """
        1) Write all input IPv6s into 'seeds' in the repo root.
        2) Run convert.py → creates seeds.npy
        3) Run main.py    → prints wildcard patterns to stdout
        4) Save stdout to patterns.txt
        """
        if not self.env_python:
            raise RuntimeError("Call initialize() first.")

        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))

        # --- 1) write seeds file ---
        seeds_file = os.path.join(repo_path, "seeds")
        with open(seeds_file, "w") as f:
            for addr in ipv6_addresses:
                try:
                    full = ipaddress.IPv6Address(addr).exploded
                except ipaddress.AddressValueError:
                    raise ValueError(f"Invalid IPv6 address: {addr!r}")
                f.write(full + "\n")
        
        print(f"Wrote {len(ipv6_addresses)} seeds to {seeds_file}")

        # --- 2) convert.py → seeds.npy ---
        run = subprocess.run(
            [self.env_python, "convert.py"],
            cwd=repo_path,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
        )

        if run.returncode != 0:
            raise RuntimeError(f"convert.py failed:\n{run.stderr}")

        # --- 3) main.py → stdout patterns ---
        run = subprocess.run(
            [self.env_python, "main.py"],
            cwd=repo_path,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
        )

        if run.returncode != 0:
            raise RuntimeError(f"main.py failed:\n{run.stderr}")

        patterns_txt = os.path.join(repo_path, "patterns.txt")
        with open(patterns_txt, "w") as f:
            f.write(run.stdout)
        
        print(f"Wrote {len(run.stdout.splitlines())} patterns to {patterns_txt}")

    def generate(self, count: int) -> list[str]:
        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        patterns_txt = os.path.join(repo_path, "patterns.txt")

        if not os.path.exists(patterns_txt):
            raise FileNotFoundError("Run train(...) before generate().")

        # load only the 32-char wildcard patterns
        with open(patterns_txt) as f:
            raw = f.read().splitlines()

        pat_re = re.compile(r"^[0-9a-f\*]{32}$")
        patterns = [L for L in raw if pat_re.match(L)]

        if not patterns:
            raise RuntimeError("No patterns found in patterns.txt")

        def sample_ip(pat: str) -> str:
            # replace each '*' by a random hex digit
            filled = "".join(
                c if c != "*" else random.choice("0123456789abcdef")
                for c in pat
            )
            # interpret as an integer, then format as exploded IPv6
            addr = ipaddress.IPv6Address(int(filled, 16))
            return addr.exploded

        # sample and return
        return [ sample_ip(random.choice(patterns)) for _ in range(count) ]
