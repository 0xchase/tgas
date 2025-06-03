import os
import subprocess
import random
import tqdm

from .base import StaticTGA, DynamicTGA, sample_ip

class SixGenTGA(StaticTGA):
    def setup(self) -> None:
        self.clone("https://github.com/ReAbout/6Gen")
        self.install_python("2.7.18")

        # Patch files
        self.patch("6gen.py", "os.path.abspath(os.path.dirname(__file__))+\"\\ips.txt\"", "\"../train/seeds.txt\"")
        self.patch("6gen.py", "os.path.abspath(os.path.dirname(__file__))+\"\\\\result.txt\"", "\"../train/result.txt\"")

    def train(self, seeds: list[str]) -> None:
        # Write seeds
        os.makedirs(self.train_dir, exist_ok=True)
        self.write_seeds(seeds, os.path.join(self.train_dir, "seeds.txt"), exploded=True, colan=False)

        # Run analysis
        self.cmd([self.python, "6gen.py"])

    def generate(self, count: int) -> list[str]:
        # Patch budget count
        # self.patch_match("6gen.py", r'budgetLimit\s*=\s*(\d+)', f'budgetLimit= {count}')

        # Read patterns
        output_file = os.path.join(self.train_dir, "result.txt")
        with open(output_file, "r") as f:
            patterns = [line.strip() for line in f if line.strip()]

            # Sample ips
            unique_ips = set()
            miniters = max(100, count // 100)
            with tqdm.tqdm(total=count, desc="Generating unique IPs", miniters=miniters) as pbar:
                while len(unique_ips) < count:
                    ip = sample_ip(random.choice(patterns))
                    if ip not in unique_ips:
                        unique_ips.add(ip)
                        pbar.update(1)

            return list(unique_ips)
