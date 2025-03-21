import os
import subprocess

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
        self.env_python = None

    def initialize(self) -> None:
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
            print(f"Running {main_script} to perform 6Forest analysis...")
            subprocess.run([
                self.env_python,
                main_script
            ], cwd=repo_path, check=True)
        else:
            print("No main.py found; skipping local 6Forest analysis script.")

        print("Training complete (logic handled in the cloned repo).")

    def generate(self, count: int) -> list[str]:
        """
        In the pure delegate approach, you might rely on the cloned repo's code
        for generating addresses. This stub can remain empty or call another script.
        """
        print("No direct generation logic here; relying on cloned repo's code for address generation.")
        return []