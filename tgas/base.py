import os
import subprocess
import ipaddress
import re
import tempfile

SETUP_DIR = os.path.abspath("setup")
SCAN_DIR = os.path.abspath("scan")

class TGA:
    def __init__(self):
        self.name = self.__class__.__name__.lower()

        # Setup directories
        self.setup_dir = os.path.join(SETUP_DIR, self.name)

        # Setup subdirectories
        self.clone_dir = os.path.join(self.setup_dir, "tga")
        self.deps_dir = os.path.join(self.setup_dir, "deps")
        self.env_dir = os.path.join(self.setup_dir, "env")
        self.setup_log = os.path.join(self.setup_dir, "setup.log")

        # Scan directory
        self.scan_dir = os.path.join(SCAN_DIR, self.name)

        self.python = None
        self.log = None

    def run_cmd(self, cmd, cwd=None):
        shell = isinstance(cmd, str)
        logf = self.get_log_file()
        with open(logf, 'a') as stdout:
            subprocess.run(cmd, cwd=cwd, shell=shell, stdout=stdout, stderr=subprocess.STDOUT, check=True)

    def clone(self, url: str) -> None:
        if not os.path.exists(self.clone_dir):
            print(f"Cloning {url} into {self.clone_dir}")
            subprocess.run(["git", "clone", url, self.clone_dir], check=True)
        else:
            print(f"Repository {self.repo_name} exists at {self.clone_dir}")

    def clean(self) -> None:
        if os.path.exists(self.setup_dir):
            print(f"Deleting setup {self.name} at {self.setup_dir}...")
            subprocess.run(["rm", "-rf", self.setup_dir], check=True)
        else:
            print(f"Nothing to clean at {self.setup_dir}")
    
    def install_python(self, version: str) -> None:
        if version.count('.') != 2:
            raise ValueError(f"Python version must be in format 'X.Y.Z' (e.g. '3.9.13'), got '{version}'")
        
        # Ensure pyenv has the requested Python version
        print(f"Ensuring pyenv has Python {version} installed...")
        subprocess.run(["pyenv", "install", "--skip-existing", version], check=True)
        
        # Get the path to the pyenv-managed Python interpreter
        pyenv_root = subprocess.run(["pyenv", "root"], capture_output=True, text=True, check=True).stdout.strip()
        python_executable = os.path.join(pyenv_root, "versions", version, "bin", "python")
        
        # Check if virtual environment exists and get its Python version
        if os.path.exists(self.env_dir):
            venv_python = os.path.join(self.env_dir, "bin", "python")
            if os.path.exists(venv_python):
                version_check = subprocess.run([venv_python, "--version"], capture_output=True, text=True)
                venv_version = (version_check.stdout or version_check.stderr).strip().split()[1]
                
                if venv_version == version:
                    print(f"Python environment exists at {self.env_dir} with version {version}")
                    self.python = venv_python
                    return
                
                print(f"Removing existing virtual environment at {self.env_dir}")
                subprocess.run(["rm", "-rf", self.env_dir], check=True)
        
        # Create the environment directory
        os.makedirs(self.env_dir, exist_ok=True)
        
        # Check Python version to determine which tool to use
        version_check = subprocess.run([python_executable, "--version"], capture_output=True, text=True)
        ver_str = version_check.stdout or version_check.stderr
        
        if "Python 2." in ver_str:
            # Use virtualenv for Python 2
            print(f"Installing virtualenv for Python 2...")
            subprocess.run([python_executable, "-m", "pip", "install", "--upgrade", "pip", "virtualenv"], check=True)
            print(f"Creating Python 2 virtual environment at {self.env_dir}...")
            subprocess.run([python_executable, "-m", "virtualenv", self.env_dir], check=True)
        else:
            # Use built-in venv for Python 3
            print(f"Creating Python 3 virtual environment at {self.env_dir}...")
            subprocess.run([python_executable, "-m", "venv", self.env_dir], check=True)
        
        # Verify the environment was created successfully
        self.python = os.path.join(self.env_dir, "bin", "python")
        if not os.path.exists(self.python):
            raise FileNotFoundError(f"Could not find '{self.python}' in the virtual environment")
        
        print(f"Python {version} environment created successfully at {self.env_dir}")
    
    def install_packages(self, deps: list[str]) -> None:
        if not self.python:
            raise RuntimeError("Python environment not installed")
            
        print(f"Installing packages into {self.python}: {deps}")
        
        # Upgrade pip first
        subprocess.run([self.python, "-m", "pip", "install", "--upgrade", "pip"], check=True)
        
        # Install the packages
        pip_cmd = [self.python, "-m", "pip", "install"]
        pip_cmd.extend(deps)
        subprocess.run(pip_cmd, check=True)
        
        print("Packages installed successfully")

    def write_seeds(self, addrs: list[str], seeds_file: str, exploded: bool = False) -> None:
        print(f"Writing {len(addrs)} seeds to {seeds_file}")

        with open(seeds_file, "w") as f:
            for addr in addrs:
                if exploded:
                    try:
                        exploded = ipaddress.IPv6Address(addr).exploded
                        f.write(exploded + "\n")
                    except ipaddress.AddressValueError:
                        raise ValueError(f"Invalid IPv6 address: {addr}")
                else:
                    f.write(addr + "\n")

class StaticTGA(TGA):
    def setup(self) -> None:
        raise NotImplementedError("")

    def train(self, seeds: list[str]) -> None:
        raise NotImplementedError("")

    def generate(self, count: int) -> list[str]:
        raise NotImplementedError("")

class DynamicTGA(TGA):
    def setup(self) -> None:
        raise NotImplementedError("")
    
    def run(self, seeds: list[str], count: int) -> list[str]:
        raise NotImplementedError("")
