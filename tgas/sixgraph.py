import os
import subprocess
import ipaddress
import random
import re
import tqdm

from .base import StaticTGA, DynamicTGA

class SixGraphTGA(StaticTGA):
    def setup(self) -> None:
        self.clone("https://github.com/Lab-ANT/6Graph")
        self.install_python("3.7.16")
        self.install_packages(["IPy", "numpy==1.21.2", "networkx"])

    def train(self, seeds: list[str]) -> None:
        print(f"Writing seeds")
        self.write_seeds(seeds, os.path.join(self.clone_dir, "seeds.txt"), exploded=True)

        # Convert seeds
        run = self.cmd([self.python, "convert.py"])
        if run.returncode != 0:
            raise RuntimeError(f"convert.py failed:\n{run.stderr}")

        # Train TGA
        run = subprocess.run([self.python, "main.py"], cwd=self.clone_dir, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        if run.returncode != 0:
            raise RuntimeError(f"main.py failed:\n{run.stderr}")

        # Write patterns
        os.makedirs(self.train_dir, exist_ok=True)
        patterns_txt = os.path.join(self.train_dir, "patterns.txt")
        with open(patterns_txt, "w+") as f:
            f.write(run.stdout)
        
        print(f"Wrote {len(run.stdout.splitlines())} patterns to {patterns_txt}")

    def generate(self, count: int) -> list[str]:
        patterns_txt = os.path.join(self.train_dir, "patterns.txt")

        # load only the 32-char wildcard patterns
        with open(patterns_txt) as f:
            raw = f.read().splitlines()

        pat_re = re.compile(r"^[0-9a-f\*]{32}$")
        patterns = [L for L in raw if pat_re.match(L)]

        if not patterns:
            raise RuntimeError("No patterns found in patterns.txt")

        def sample_ip(pat: str) -> str:
            # replace each '*' by a random hex digit
            filled = "".join(c if c != "*" else random.choice("0123456789abcdef") for c in pat)
            addr = ipaddress.IPv6Address(int(filled, 16))
            return addr.exploded

        unique_ips = set()
        miniters = max(100, count // 100)
        with tqdm.tqdm(total=count, desc="Generating unique IPs", miniters=miniters) as pbar:
            while len(unique_ips) < count:
                ip = sample_ip(random.choice(patterns))
                if ip not in unique_ips:
                    unique_ips.add(ip)
                    pbar.update(1)

        return list(unique_ips)
