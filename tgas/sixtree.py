import os
import subprocess
import sys
import random

from .base import TGA

TEMP_FILE_NAME = "temp_addresses.txt"

# TODO: Requires real-time scan feedback
class SixTreeTGA(TGA):
    """
    A single-file approach to 6GAN TGA:
     1) Clone a repo into a subdirectory with its own venv.
     2) Install dependencies in that venv (tensorflow, gensim, scikit-learn, ipaddress).
     3) Defer all logic (train.py, etc.) to the cloned repository.
     4) Uses absolute paths for everything.
    """
    def __init__(self, github_url: str, clone_directory: str = "repos"):
        super().__init__(github_url, clone_directory)

    def initialize(self) -> None:
        # First clone the repository
        self.clone()
        # Then initialize Python environment
        self._initialize_python("3.9.1", [])

    def train(self, ipv6_addresses: list[str]) -> None:
        """
        1) Writes IPv6 addresses to 'source_file' in the repo directory.
        2) Invokes 'train.py' to perform training.
        """
        if not self.env_python:
            raise RuntimeError("Environment not initialized. Call initialize() first.")

        # Write addresses to temp file in repo
        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        temp_file = os.path.join(repo_path, TEMP_FILE_NAME)
        with open(temp_file, "w") as f:
            for addr in ipv6_addresses:
                f.write(addr + "\n")

        print("Wrote seed addresses to temporary file")

    def generate(self, count: int) -> list[str]:
        """
        Placeholder method to generate new IPv6 addresses.
        """

        print("Generating addresses...")

        # Import the necessary modules
        sys.path.append(os.path.join(self.clone_directory, self.repo_name))
        from AddrsToSeq import InputAddrs, SeqToAddrs

        # Read addresses from temp file in repo
        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        temp_file = os.path.join(repo_path, TEMP_FILE_NAME)

        # Exception if temp file doesn't exist
        if not os.path.exists(temp_file):
            raise FileNotFoundError(f"Seed file {temp_file} not found, run train() first")

        # Convert addresses to sequence
        input_addrs = InputAddrs(temp_file, beta=16)
        targets = SeqToAddrs(input_addrs)

        # Ensure the number of targets is greater than the requested count
        if len(targets) < count:
            raise ValueError(f"Requested {count} addresses, but only {len(targets)} targets available")

        # Sample a random subset of the targets
        subset = random.sample(targets, count)

        return subset
