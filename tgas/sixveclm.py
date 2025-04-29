import os
import subprocess
import ipaddress
import random

from .base import TGA

class SixVecLMTGA(TGA):
    def setup(self) -> None:
        self.clone("https://github.com/CuiTianyu961030/6VecLM")
        self.install_python("3.7.16")
        self.install_packages(["torch==1.3.1", "torchvision", "torchaudio", "gensim==3.6.0", "scikit-learn", "torchsummary", "matplotlib", "seaborn"])

        #self._initialize_python(
        #    "3.7.16",
        #    [
        #        "torch",
        #        "gensim",
        #        "scikit-learn",
        #        "numpy",
        #        "pandas",
        #        "matplotlib",
        #        "seaborn",
        #        "torchsummary",
        #    ],
        #)

    def train(self, seeds: list[str]) -> None:
        # Write seeds
        seeds_file = os.path.join(self.clone_dir, "data", "public_dataset", "sample_addresses.txt")
        self.write_seeds(seeds, seeds_file, exploded=True)

        # Process data
        print("Processing data")
        run = self.cmd([self.env_python, "data_processing.py"])
        if run.returncode != 0:
            raise RuntimeError(f"data_processing.py failed:\n{run.stderr}")

        # Convert to vectors
        print("Converting to vectors")
        run = self.cmd([self.env_python, "ipv62vec.py"])
        if run.returncode != 0:
            raise RuntimeError(f"ipv62vec.py failed:\n{run.stderr}")
        
        # Transformer
        run = self.cmd([self.env_python, "ipv6_transformer.py"])
        if run.returncode != 0:
            raise RuntimeError(f"ipv6_transformer.py failed:\n{run.stderr}")

    def generate(self, count: int) -> list[str]:
        cand_file = os.path.join(self.clone_dir, "data", "generation_data", "candidate_s6_e10_t0015.txt")
        if not os.path.exists(cand_file):
            raise FileNotFoundError("Run train(...) before generate().")

        with open(cand_file) as f:
            candidates = [line.strip() for line in f if line.strip()]

        if not candidates:
            raise RuntimeError("No candidates found in generation_data.")

        # sample with replacement
        return [random.choice(candidates) for _ in range(count)]
