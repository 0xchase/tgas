import os
import subprocess
import ipaddress
import re
import tqdm
import random
import tempfile

import tqdm._tqdm

SETUP_DIR = os.path.abspath("setup")
RUN_DIR = os.path.abspath("run")

class TGA:
    def __init__(self):
        self.name = self.__class__.__name__.lower()

        # Setup directories
        self.setup_dir = os.path.join(SETUP_DIR, self.name)

        # Setup subdirectories
        self.clone_dir = os.path.join(self.setup_dir, "tga")
        self.deps_dir = os.path.join(self.setup_dir, "deps")
        self.env_dir = os.path.join(self.setup_dir, "env")
        self.train_dir = os.path.join(self.setup_dir, "train")

        # Run directory
        self.run_dir = os.path.join(RUN_DIR, self.name)

        # Setup logs
        self.log = None

        # Other
        self.python = os.path.join(self.env_dir, "bin", "python")

        os.makedirs(SETUP_DIR, exist_ok=True)
        os.makedirs(RUN_DIR, exist_ok=True)
        os.makedirs(self.setup_dir, exist_ok=True)
        os.makedirs(self.run_dir, exist_ok=True)

    def cmd(self, cmd):
        env = os.environ.copy()
        venv_bin = os.path.join(self.env_dir, "bin")
        env["PATH"] = venv_bin + os.pathsep + env.get("PATH", "")

        shell = isinstance(cmd, str)
        with open(self.log, 'a+') as stdout:
            return subprocess.run(cmd, cwd=self.clone_dir, shell=shell, stdout=stdout, stderr=subprocess.STDOUT, check=True, env=env)

    def clone(self, url: str) -> None:
        if not os.path.exists(self.clone_dir):
            print(f"Cloning {url} into {self.clone_dir}")
            os.makedirs(self.clone_dir, exist_ok=True)
            self.cmd(["git", "clone", url, self.clone_dir])
        else:
            print(f"Repository {url} exists at {self.clone_dir}")

    def clean(self) -> None:
        if os.path.exists(self.setup_dir):
            print(f"Deleting setup {self.name} at {self.setup_dir}...")
            self.cmd(["rm", "-rf", self.setup_dir])
        else:
            print(f"Nothing to clean at {self.setup_dir}")
    
    def install_python(self, version: str) -> None:
        if version.count('.') != 2:
            raise ValueError(f"Python version must be in format 'X.Y.Z' (e.g. '3.9.13'), got '{version}'")
        
        # Ensure pyenv has the requested Python version
        print(f"Ensuring pyenv has Python {version} installed...")
        subprocess.run(["pyenv", "install", "--skip-existing", version], check=True)

        self.cmd(["pyenv", "local", version])
        
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
                    return
                
                print(f"Removing existing virtual environment at {self.env_dir}")
                self.cmd(["rm", "-rf", self.env_dir])
        
        # Create the environment directory
        os.makedirs(self.env_dir, exist_ok=True)
        
        # Check Python version to determine which tool to use
        version_check = subprocess.run([python_executable, "--version"], capture_output=True, text=True)
        ver_str = version_check.stdout or version_check.stderr
        
        if "Python 2." in ver_str:
            # Use virtualenv for Python 2
            print(f"Installing virtualenv for Python 2...")
            self.cmd([python_executable, "-m", "pip", "install", "--upgrade", "pip", "virtualenv"])
            print(f"Creating Python 2 virtual environment at {self.env_dir}...")
            self.cmd([python_executable, "-m", "virtualenv", self.env_dir])
        else:
            # Use built-in venv for Python 3
            print(f"Creating Python 3 virtual environment at {self.env_dir}...")
            self.cmd([python_executable, "-m", "venv", self.env_dir])
        
        # Verify the environment was created successfully
        if not os.path.exists(self.python):
            raise FileNotFoundError(f"Could not find '{self.python}' in the virtual environment")
        
        print(f"Python {version} environment created at {self.env_dir}")
    
    def install_packages(self, deps: list[str]) -> None:
        if not os.path.exists(self.python):
            raise RuntimeError("Python environment not installed")
            
        print(f"Installing packages into {self.python}: {deps}")
        
        # Upgrade pip first
        self.cmd([self.python, "-m", "pip", "install", "--upgrade", "pip"])
        
        # Install the packages
        pip_cmd = [self.python, "-m", "pip", "install"]
        pip_cmd.extend(deps)
        self.cmd(pip_cmd)
        
        print("Packages installed successfully")

    def write_seeds(self, addrs: list[str], seeds_file: str, colan=True, exploded: bool = False) -> None:
        print(f"Writing {len(addrs)} seeds to {seeds_file}")

        miniters = max(100, len(addrs) // 100)
        with open(seeds_file, "w+") as f:
            for addr in tqdm.tqdm(addrs, desc="Writing seeds", miniters=miniters):
                if exploded:
                    try:
                        exploded = ipaddress.IPv6Address(addr).exploded
                        if not colan:
                            exploded = exploded.replace(":", "")
                        f.write(exploded + "\n")
                    except ipaddress.AddressValueError:
                        raise ValueError(f"Invalid IPv6 address: {addr}")
                else:
                    if not colan:
                        addr = addr.replace(":", "")
                    f.write(addr + "\n")

    def patch(self, file_name: str, old_text: str, new_text: str) -> None:
         path = os.path.join(self.clone_dir, file_name)
         
         # Read original
         with open(path, 'r', encoding='utf-8') as f:
             content = f.read()
         
         # Replace
         patched = content.replace(old_text, new_text)
         
         # Write back only if changed
         if patched != content:
             print(f"Patching {file_name} to replace {old_text} with {new_text}")
             with open(path, 'w', encoding='utf-8') as f:
                 f.write(patched)

    def patch_match(self, file_name: str, pattern: str, replacement: str) -> None:
        path = os.path.join(self.clone_dir, file_name)
        
        # Read original
        with open(path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Replace using regex
        patched = re.sub(pattern, replacement, content, flags=re.MULTILINE)
        
        # Write back only if changed
        if patched != content:
            print(f"Patching {file_name} to replace pattern '{pattern}' with '{replacement}'")
            with open(path, 'w', encoding='utf-8') as f:
                f.write(patched)

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
    
    def run(self, seeds: list[str], budget: int) -> list[str]:
        raise NotImplementedError("")

def sample_ip(pat: str) -> str:
    # replace each '*' or '?' with a random hex digit
    filled = "".join(c if c != "*" and c != "?" else random.choice("0123456789abcdef") for c in pat)
    addr = ipaddress.IPv6Address(int(filled, 16))
    return addr.exploded