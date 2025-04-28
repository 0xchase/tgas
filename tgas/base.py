import os
import subprocess

class TGA:
    """
    Base class representing a target generation algorithm.
    """
    def __init__(self, github_url: str, clone_directory: str = "repos"):
        # Use an absolute path for the clone directory
        self.clone_directory = os.path.abspath(clone_directory)
        self.github_url = github_url
        self.repo_name = self._extract_repo_name()
        self._python_version = None  # Track the Python version used for initialization

    def _extract_repo_name(self) -> str:
        repo_name = self.github_url.split('/')[-1]
        if repo_name.endswith('.git'):
            repo_name = repo_name[:-4]
        return repo_name

    def _initialize_python(self, python_version: str, deps: list[str]) -> None:
        if python_version.count('.') != 2:
            raise ValueError(f"Python version must be in format 'X.Y.Z' (e.g. '3.9.13'), got '{python_version}'")
        
        # Check if Python version has changed
        if self._python_version != python_version:
            print(f"Python version changed from {self._python_version} to {python_version}. Recreating virtual environment...")
            self._python_version = python_version
            # Force recreation of virtual environment
            env_path = os.path.join(self.clone_directory, self.repo_name, "venv")
            if os.path.exists(env_path):
                print(f"Removing existing virtual environment at {env_path}...")
                subprocess.run(["rm", "-rf", env_path], check=True)
        
        print(f"Ensuring pyenv has Python {python_version} installed...")
        subprocess.run(["pyenv", "install", "--skip-existing", python_version], check=True)

        # Build the path to the pyenv-managed Python interpreter
        pyenv_root = subprocess.run(["pyenv", "root"], capture_output=True, text=True, check=True).stdout.strip()
        python_executable = os.path.join(pyenv_root, "versions", python_version, "bin", "python")

        # Create venv directory in the repo
        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        env_path = os.path.join(repo_path, "venv")

        # Check if the virtual environment already exists
        if os.path.exists(env_path):
            print(f"Virtual environment already exists at {env_path}.")
        else:
            # Distinguish Python 2 vs. 3
            # We call `python_executable --version` to see major version
            version_check = subprocess.run([python_executable, "--version"], capture_output=True, text=True)
            # Typically returns "Python 3.9.13" or "Python 2.7.18"
            ver_str = version_check.stdout or version_check.stderr
            # Or parse using sys.version_info in a separate command

            if "Python 2." in ver_str:
                # Use virtualenv
                # 1) Ensure 'virtualenv' is installed in that python
                subprocess.run([python_executable, "-m", "pip", "install", "--upgrade", "pip", "virtualenv"], check=True)
                print(f"Creating Python2.7 virtual environment at {env_path} with virtualenv...")
                subprocess.run([python_executable, "-m", "virtualenv", env_path], check=True)
            else:
                # Use built-in venv
                print(f"Creating virtual environment at {env_path} with {python_executable} -m venv ...")
                subprocess.run([python_executable, "-m", "venv", env_path], check=True)

        # Path to the newly created environment's python
        self.env_python = os.path.join(env_path, "bin", "python")
        if not os.path.exists(self.env_python):
            raise FileNotFoundError(f"Could not find '{self.env_python}' in the virtual environment.")

        # Add tqdm to the dependencies
        deps = deps + ["tqdm"]
        print(f"Installing dependencies into {self.env_python}: {deps}")
        pip_cmd = [self.env_python, "-m", "pip", "install", "--upgrade", "pip"] + deps
        subprocess.run(pip_cmd, check=True)
        print("Environment ready.")

    def clone(self) -> None:
        """
        Clones the GitHub repository into a specified subdirectory (unless it already exists).
        """
        clone_path = os.path.join(self.clone_directory, self.repo_name)
        if not os.path.exists(clone_path):
            print(f"Cloning {self.github_url} into {clone_path}...")
            subprocess.run(["git", "clone", self.github_url, clone_path], check=True)
        else:
            print(f"Repository {self.repo_name} already exists at {clone_path}. Skipping clone.")

    def initialize(self) -> None:
        """
        Placeholder method to initialize the cloned repo.
        """
        pass

    def train(self, ipv6_addresses: list[str]) -> None:
        """
        Placeholder method to train using a list of IPv6 addresses.
        """

        print("Training the model...")

        pass

    def generate(self, count: int) -> list[str]:
        """
        Placeholder method to generate new IPv6 addresses.
        """

        print("Generating addresses...")

        return []

    def clean(self) -> None:
        """
        Deletes the cloned repository if it exists.
        """
        clone_path = os.path.join(self.clone_directory, self.repo_name)
        if os.path.exists(clone_path):
            print(f"Deleting repository {self.repo_name} at {clone_path}...")
            subprocess.run(["rm", "-rf", clone_path], check=True)
        else:
            print(f"Repository {self.repo_name} does not exist at {clone_path}. Nothing to clean.")