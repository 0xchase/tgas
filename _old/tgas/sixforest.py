import os
import subprocess
import random

from .base import StaticTGA, DynamicTGA

class SixForestTGA(StaticTGA):
    def setup(self) -> None:
        self.clone("https://github.com/Lab-ANT/6Forest")
        self.install_python("3.9.6")
        self.install_packages(["numpy==1.21.2", "IPy==1.1"])

    def train(self, seeds: list[str]) -> None:
        # Write seeds
        os.makedirs(self.train_dir, exist_ok=True)
        self.write_seeds(seeds, os.path.join(self.train_dir, "seeds.txt"))

        # Convert seeds into seeds.npy
        print(f"Converting seeds into seeds.npy")
        self.cmd([self.python, os.path.join(self.clone_dir, "convert.py")])

        # Main logic
        main_script = os.path.join(self.clone_dir, "main.py")
        output_file = os.path.join(self.train_dir, "main_output.txt")
        print(f"Running 6Forest analysis and redirecting output to {output_file}")
        with open(output_file, "w") as f:
            subprocess.run([self.python, main_script], cwd=self.clone_dir, check=True, stdout=f, stderr=subprocess.STDOUT)

    def generate(self, count: int) -> list[str]:
        output_file = os.path.join(self.train_dir, "main_output.txt")

        # Read partial addresses
        partial_addresses = []
        with open(output_file, "r") as f:
            lines = [line.strip() for line in f if line.strip()]

        # Extract the first address directly below each "Region" divider
        for i, line in enumerate(lines):
            if "********Region**********" in line:
                if i + 1 < len(lines) and '*' in lines[i + 1]:
                    partial_addresses.append(lines[i + 1])

        # Generate full IP addresses
        full_addresses = []
        while len(full_addresses) < count:
            for partial in partial_addresses:
                if len(full_addresses) >= count:
                    break
                full_address = ''.join(random.choice('0123456789abcdef') if c == '*' else c for c in partial)
                formatted_address = ':'.join(full_address[i:i+4] for i in range(0, len(full_address), 4))
                full_addresses.append(formatted_address)
                
        return full_addresses[:count]