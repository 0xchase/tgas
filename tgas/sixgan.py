import os
import subprocess

from .base import TGA

class SixGANTGA(TGA):
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
        self._initialize_python("3.9.6", ["tensorflow", "gensim", "scikit-learn", "ipaddress"])

    def train(self, ipv6_addresses: list[str]) -> None:
        """
        1) Writes IPv6 addresses to 'source_file' in the repo directory.
        2) Invokes 'train.py' to perform training.
        """
        if not self.env_python:
            raise RuntimeError("Environment not initialized. Call initialize() first.")

        print(f"Training the SixGAN model in repo '{self.repo_name}'...")

        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        source_file = os.path.join(repo_path, "data/source_data/responsive-addresses.txt")

        # Step 1: Write source data
        with open(source_file, "w") as f:
            for addr in ipv6_addresses:
                f.write(addr + "\n")

        # Step 2: train.py for 6GAN logic
        train_script = os.path.join(repo_path, "train.py")
        if os.path.exists(train_script):
            print(f"Running {train_script} to perform 6GAN training...")
            subprocess.run([
                self.env_python,
                train_script
            ], cwd=repo_path, check=True)
        else:
            print("No train.py found; skipping local 6GAN training script.")

        print("Training complete (logic handled in the cloned repo).")

    def generate(self, count: int) -> list[str]:
        """
        Placeholder method to generate new IPv6 addresses.
        """
        print("Generating addresses...")

        # This method should be implemented to parse the output of the 6GAN training
        # and generate IPv6 addresses accordingly.

        return []
