import os
import subprocess
import random

from .base import TGA

class SixForestTGA(TGA):
    """
    A single-file approach to 6Forest TGA:
     1) Clone a repo into a subdirectory with its own venv.
     2) Install dependencies in that venv (numpy>=1.21.2, IPy>=1.1).
     3) Defer all logic (convert.py, main.py, etc.) to the cloned repository.
     4) Uses absolute paths for everything.
    """
    def __init__(self, github_url: str, clone_directory: str = "repos"):
        super().__init__(github_url, clone_directory)

    def initialize(self) -> None:
        # First clone the repository
        self.clone()
        # Then initialize Python environment
        self._initialize_python("3.9.6", ["numpy==1.21.2", "IPy==1.1"])

    def train(self, ipv6_addresses: list[str]) -> None:
        """
        1) Writes IPv6 addresses to 'seeds' in the repo directory.
        2) Invokes 'convert.py' if present, then (optionally) invokes 'main.py'
           or another script from the cloned repo to do the actual training/analysis.
        """
        if not self.env_python:
            raise RuntimeError("Environment not initialized. Call initialize() first.")

        print(f"Training the SixForest model in repo '{self.repo_name}'...")

        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        seeds_file = os.path.join(repo_path, "seeds")

        # Step 1: Write seeds
        with open(seeds_file, "w") as f:
            for addr in ipv6_addresses:
                f.write(addr + "\n")

        # Step 2: convert.py => seeds.npy if present
        convert_script = os.path.join(repo_path, "convert.py")
        if os.path.exists(convert_script):
            print(f"Running {convert_script} to generate seeds.npy...")
            subprocess.run([
                self.env_python,
                convert_script
            ], cwd=repo_path, check=True)
        else:
            print("No convert.py found; skipping conversion step.")

        # (Optional) Step 3: main.py for 6Forest logic
        main_script = os.path.join(repo_path, "main.py")
        if os.path.exists(main_script):
            output_file = os.path.join(repo_path, "main_output.txt")
            print(f"Running {main_script} to perform 6Forest analysis and redirecting output to {output_file}...")
            with open(output_file, "w") as f:
                subprocess.run([
                    self.env_python,
                    main_script
                ], cwd=repo_path, check=True, stdout=f, stderr=subprocess.STDOUT)
        else:
            print("No main.py found; skipping local 6Forest analysis script.")

        print("Training complete (logic handled in the cloned repo).")

    def generate(self, count: int) -> list[str]:
        """
        Parses the output file from the training process to obtain the first address directly below the "Region" divider,
        and generates the desired number of full IP addresses by replacing '*' with random values.
        The generated addresses are formatted with colons in the appropriate places.
        """
        print("Generating addresses from the output file...")

        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        output_file = os.path.join(repo_path, "main_output.txt")

        if not os.path.exists(output_file):
            print(f"Output file {output_file} does not exist. Cannot generate addresses.")
            return []

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
                # Format the address with colons
                formatted_address = ':'.join(full_address[i:i+4] for i in range(0, len(full_address), 4))
                full_addresses.append(formatted_address)

        return full_addresses[:count]