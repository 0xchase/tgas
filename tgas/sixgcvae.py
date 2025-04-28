import os
import subprocess
import sys
import runpy
import ipaddress

from .base import TGA

class SixGcVaeTGA(TGA):
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
        self._initialize_python("3.8.12", ["tensorflow", "keras", "scikit-learn"])

    def train(self, ipv6_addresses: list[str]) -> None:
        """
        1) Writes IPv6 addresses to 'source_file' in the repo directory.
        2) Invokes 'train.py' to perform training.
        """
        if not self.env_python:
            raise RuntimeError("Environment not initialized. Call initialize() first.")
        
        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))

        # Prepare processed data directory & file
        processed_dir = os.path.join(repo_path, 'data', 'processed_data')
        os.makedirs(processed_dir, exist_ok=True)
        data_file = os.path.join(processed_dir, 'data.txt')

        # Write addresses to data file
        with open(data_file, 'w') as f:
            print("Writing formatted addresses to data file...")
            for addr in ipv6_addresses:
                try:
                    full = ipaddress.IPv6Address(addr).exploded 
                    print(full)
                except ipaddress.AddressValueError:
                    raise ValueError(f"Invalid IPv6 address: '{addr}'")

                # strip colons, lowercase, write one 32-hex string per line
                hex_str = full.replace(":", "").lower()
                if len(hex_str) != 32:
                    raise ValueError(f"Address '{full}' is not 32 hex characters long")
                f.write(hex_str + "\n")
        
        # Prepare models directory
        models_dir = os.path.join(repo_path, 'models')
        os.makedirs(models_dir, exist_ok=True)

        # Call the existing training script
        #    gcnn_vae.py’s run_model() will load data/processed_data/data.txt,
        #    train the VAE, and save weights to models/gcnn_vae.model
        training_script = os.path.join(repo_path, 'gcnn_vae.py')
        print("Training the VAE...")
        result = subprocess.run(
            [self.env_python, training_script],
            cwd=repo_path,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )

        if result.returncode != 0:
            # Propagate any error output for debugging
            raise RuntimeError(
                f"Training failed (exit code {result.returncode}):\n"
                f"{result.stderr}"
            )
        
        print(result.stdout)
        print("Training complete — weights saved to models/gcnn_vae.model")

    def generate(self, count: int) -> list[str]:
        """
        Placeholder method to generate new IPv6 addresses.
        """

        print("Generating addresses...")

        sys.path.append(os.path.join(self.clone_directory, self.repo_name))
        import generation

        generation.generation_number = count

        # 4) Re-run generation.py as if it were __main__
        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        gen_py = os.path.join(repo_path, 'generation.py')
        runpy.run_path(gen_py, run_name='__main__')

        # 5) Read back the results
        out_dir  = os.path.join(repo_path, 'data', 'generated_data')
        # by default generation.py writes to "6gcvae_generation.txt"
        out_file = os.path.join(out_dir, '6gcvae_generation.txt')
        if not os.path.exists(out_file):
            raise FileNotFoundError(f"Expected output at {out_file}")

        with open(out_file, 'r') as f:
            # strip newlines, ensure colon-format
            return [line.strip() for line in f]
