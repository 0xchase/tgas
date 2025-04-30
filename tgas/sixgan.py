import os
import subprocess
import re
import glob

from .base import StaticTGA, DynamicTGA

class SixGANTGA(StaticTGA):
    def setup(self) -> None:
        self.clone("https://github.com/CuiTianyu961030/6GAN")
        self.install_python("3.7.16")
        self.install_packages(["scikit-learn"])
        self.install_packages(["tensorflow-gpu==1.15.5", "gensim==3.8.3", "pandas", "numpy", "ipaddress"])
        self.install_packages(["protobuf==3.12.2"])

        # Build the dependency
        os.makedirs(self.deps_dir, exist_ok=True)

        dest = os.path.join(self.deps_dir, "ipv6toolkit")
        if not os.path.exists(dest):
            url = "https://github.com/fgont/ipv6toolkit"
            self.cmd(["git", "clone", url, dest])
        
        if not os.path.exists(os.path.join(dest, "addr6")):
            subprocess.run(["make", "addr6"], cwd=dest, stdout=self.log)
        
        # Patch dependency location
        self.patch("classifier.py", "../../Tools/ipv6toolkit/addr6", "../deps/ipv6toolkit/addr6")

    def train(self, seeds: list[str]) -> None:
        # Write seeds
        source_file = os.path.join(self.clone_dir, "data/source_data/responsive-addresses.txt")
        self.write_seeds(seeds, source_file)

        # Train the model
        print(f"Training 6GAN model")
        self.cmd([self.python, os.path.join(self.clone_dir, "train.py")])

    def generate(self, count: int) -> list[str]:
        candidate_dir = os.path.join(self.clone_dir, "data/candidate_set")
        pattern = os.path.join(candidate_dir, "candidate_generator_*_epoch_*.txt")
        files = glob.glob(pattern)
        if not files:
            raise RuntimeError("No candidate files found; run train() to generate data first")
        
        # Select the latest epoch file based on the epoch number in the filename
        latest_file = max(files, key=lambda f: int(re.search(r'_epoch_(\d+)\\.txt', os.path.basename(f)).group(1)))

        print(f"Reading generated addresses from {latest_file}...")
        addresses: list[str] = []
        with open(latest_file, "r") as f:
            for line in f:
                line = line.strip()
                if line:
                    addresses.append(line)
        return addresses[:count]
