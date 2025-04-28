import os
import subprocess
import re
import glob

from .base import TGA

class SixGANTGA(TGA):
    def __init__(self, github_url: str, clone_directory: str = "repos"):
        super().__init__(github_url, clone_directory)

    def initialize(self) -> None:
        self.clone()

        self._initialize_python(
            "3.10.1",
            ["tensorflow", "tensorflow-addons", "gensim", "scikit-learn", "ipaddress", "pandas"],
        )

        # Initialize the patched files
        self._patch_initialize(["train.py", "classifier.py", "generator.py"])

        # Patch tensorflow to use tensorflow 1.x
        for file in ["train.py", "generator.py"]:
            # Patch API compatibility
            imports = (
                "import tensorflow.compat.v1 as tf\n"
                "import tensorflow_addons as tfa\n"
                "tf.disable_v2_behavior()\n"
                "tf.contrib = type('contrib', (), {})()\n"
                "tf.contrib.rnn = tf.compat.v1.nn.rnn_cell\n"
                "tf.contrib.seq2seq = tfa.seq2seq\n"
            )

            self._patch_replace(file, "import tensorflow as tf", imports)

        # Clone and build the ipv6toolkit repo
        TGA("https://github.com/fgont/ipv6toolkit").clone()
        subprocess.run(["make", "addr6"], cwd="repos/ipv6toolkit", check=True)

        # Other patches
        self._patch_replace("generator.py", "tf.random_normal(", "tf.random.normal(")
        self._patch_replace("classifier.py", "../../Tools/ipv6toolkit/addr6", "../ipv6toolkit/addr6")

    def train(self, ipv6_addresses: list[str]) -> None:
        """
        1) Writes IPv6 addresses to 'data/source_data/responsive-addresses.txt' in the repo directory.
        2) Invokes 'train.py' to perform training.
        """
        if not self.env_python:
            raise RuntimeError("Environment not initialized. Call initialize() first.")

        print(f"Training the SixGAN model in repo '{self.repo_name}'...")

        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        source_file = os.path.join(repo_path, "data/source_data/responsive-addresses.txt")

        os.makedirs(os.path.dirname(source_file), exist_ok=True)
        with open(source_file, "w") as f:
            for addr in ipv6_addresses:
                f.write(addr + "\n")

        train_script = os.path.join(repo_path, "train.py")
        if os.path.exists(train_script):
            print(f"Running {train_script} to perform 6GAN training...")
            subprocess.run(
                [self.env_python, train_script],
                cwd=repo_path,
                check=True,
            )
        else:
            print("No train.py found; skipping training script.")

        print("Training complete (handled by the cloned repository).")

    def generate(self, count: int) -> list[str]:
        """
        Generates IPv6 addresses by reading the latest candidate set produced by train.py.

        Returns up to `count` addresses.
        """
        if not self.env_python:
            raise RuntimeError("Environment not initialized. Call initialize() first.")

        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        candidate_dir = os.path.join(repo_path, "data/candidate_set")
        pattern = os.path.join(candidate_dir, "candidate_generator_*_epoch_*.txt")
        files = glob.glob(pattern)
        if not files:
            raise RuntimeError(
                "No candidate files found; please run train() to generate data first."
            )
        # Select the latest epoch file based on the epoch number in the filename
        latest_file = max(
            files,
            key=lambda f: int(re.search(r'_epoch_(\d+)\\.txt', os.path.basename(f)).group(1)),
        )
        print(f"Reading generated addresses from {latest_file}...")
        addresses: list[str] = []
        with open(latest_file, "r") as f:
            for line in f:
                line = line.strip()
                if line:
                    addresses.append(line)
        return addresses[:count]
