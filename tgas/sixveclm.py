import os
import subprocess
import ipaddress
import random

from .base import TGA

class SixVecLMTGA(TGA):
    def __init__(self, github_url: str, clone_directory: str = "repos"):
        super().__init__(github_url, clone_directory)

    def initialize(self) -> None:
        self.clone()

        self._initialize_python(
            "3.7.16",
            [
                "torch",
                "gensim",
                "scikit-learn",
                "numpy",
                "pandas",
                "matplotlib",
                "seaborn",
                "torchsummary",
            ],
        )

    def train(self, ipv6_addresses: list[str]) -> None:
        """
        1) Overwrite data/public_dataset/sample_addresses.txt with our seeds
        2) Run data_processing.py → creates processed_data/*.txt
        3) Run ipv62vec.py      → trains Word2Vec, produces models & plots
        4) Run ipv6_transformer.py → trains Transformer & writes 
           candidates to data/generation_data/candidate_s6_e10_t0015.txt
        """
        if not self.env_python:
            raise RuntimeError("Call initialize() first.")

        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))

        # --- 1) write seed file ---
        seeds_file = os.path.join(
            repo_path, "data", "public_dataset", "sample_addresses.txt"
        )
        with open(seeds_file, "w") as f:
            for addr in ipv6_addresses:
                try:
                    exploded = ipaddress.IPv6Address(addr).exploded
                except ipaddress.AddressValueError:
                    raise ValueError(f"Invalid IPv6 address: {addr!r}")
                f.write(exploded + "\n")
        print(f"Wrote {len(ipv6_addresses)} seeds to {seeds_file}")

        # --- 2) data_processing.py ---
        run = subprocess.run(
            [self.env_python, "data_processing.py"],
            cwd=repo_path,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        if run.returncode != 0:
            raise RuntimeError(f"data_processing.py failed:\n{run.stderr}")
        print("Data processing complete.")

        # --- 3) ipv62vec.py ---
        run = subprocess.run(
            [self.env_python, "ipv62vec.py"],
            cwd=repo_path,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        if run.returncode != 0:
            raise RuntimeError(f"ipv62vec.py failed:\n{run.stderr}")
        print("Word2Vec model trained & clustering done.")

        # --- 4) ipv6_transformer.py ---
        run = subprocess.run(
            [self.env_python, "ipv6_transformer.py"],
            cwd=repo_path,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        if run.returncode != 0:
            raise RuntimeError(f"ipv6_transformer.py failed:\n{run.stderr}")
        print("Transformer trained and candidate file generated.")

    def generate(self, count: int) -> list[str]:
        """
        Reads the generated candidate file and samples `count` 
        IPv6 addresses (with replacement).
        """
        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        cand_file = os.path.join(
            repo_path,
            "data",
            "generation_data",
            "candidate_s6_e10_t0015.txt",
        )

        if not os.path.exists(cand_file):
            raise FileNotFoundError("Run train(...) before generate().")

        with open(cand_file) as f:
            candidates = [line.strip() for line in f if line.strip()]

        if not candidates:
            raise RuntimeError("No candidates found in generation_data.")

        # sample with replacement
        return [random.choice(candidates) for _ in range(count)]
