import os
import subprocess
import re
import tempfile

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

    def _install_dependencies(self, deps: list[str], prefer_binary: bool = False, skip_deps: bool = False) -> None:
        if not hasattr(self, 'env_python') or not os.path.exists(self.env_python):
            raise RuntimeError("Virtual environment Python not found. Call _initialize_python first.")
            
        print(f"Installing dependencies into {self.env_python}: {deps}")
        pip_cmd = [self.env_python, "-m", "pip", "install", "--upgrade", "pip"]
        if prefer_binary:
            pip_cmd.append("--prefer-binary")
        if skip_deps:
            pip_cmd.append("--no-deps")
        pip_cmd.extend(deps)
        subprocess.run(pip_cmd, check=True)
        print("Dependencies installed successfully.")

    def _initialize_python(self, python_version: str, deps: list[str], rosetta: bool = False) -> None:
        if python_version.count('.') != 2:
            raise ValueError(f"Python version must be in format 'X.Y.Z' (e.g. '3.9.13'), got '{python_version}'")
        
        # Create venv directory in the repo
        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        env_path = os.path.join(repo_path, "venv")
        
        # Check if virtual environment exists and get its Python version and architecture
        if os.path.exists(env_path):
            venv_python = os.path.join(env_path, "bin", "python")
            if os.path.exists(venv_python):
                # Get the version from the venv's Python
                version_check = subprocess.run([venv_python, "--version"], capture_output=True, text=True)
                venv_version = (version_check.stdout or version_check.stderr).strip().split()[1]
                
                # Check the architecture of the venv's Python
                arch_check = subprocess.run(["file", venv_python], capture_output=True, text=True)
                is_x86 = "x86_64" in arch_check.stdout
                
                # Recreate venv if version or architecture doesn't match
                if venv_version != python_version or is_x86 != rosetta:
                    reason = []
                    if venv_version != python_version:
                        reason.append(f"version mismatch (venv has {venv_version}, requested {python_version})")
                    if is_x86 != rosetta:
                        current_arch = "x86_64" if is_x86 else "native"
                        requested_arch = "x86_64" if rosetta else "native"
                        reason.append(f"architecture mismatch (venv is {current_arch}, requested {requested_arch})")
                    
                    print(f"Virtual environment needs recreation: {', '.join(reason)}")
                    print(f"Removing existing virtual environment at {env_path}...")
                    subprocess.run(["rm", "-rf", env_path], check=True)
        
        print(f"Ensuring pyenv has Python {python_version} installed...")
        subprocess.run(["pyenv", "install", "--skip-existing", python_version], check=True)

        # Build the path to the pyenv-managed Python interpreter
        pyenv_root = subprocess.run(["pyenv", "root"], capture_output=True, text=True, check=True).stdout.strip()
        python_executable = os.path.join(pyenv_root, "versions", python_version, "bin", "python")

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
                if rosetta:
                    subprocess.run(["arch", "-x86_64", python_executable, "-m", "virtualenv", env_path], check=True)
                else:
                    subprocess.run([python_executable, "-m", "virtualenv", env_path], check=True)
            else:
                # Use built-in venv
                print(f"Creating virtual environment at {env_path} with {python_executable} -m venv ...")
                if rosetta:
                    subprocess.run(["arch", "-x86_64", python_executable, "-m", "venv", env_path], check=True)
                else:
                    subprocess.run([python_executable, "-m", "venv", env_path], check=True)

        # Path to the newly created environment's python
        self.env_python = os.path.join(env_path, "bin", "python")
        if not os.path.exists(self.env_python):
            raise FileNotFoundError(f"Could not find '{self.env_python}' in the virtual environment.")

        # Add tqdm to the dependencies and install them
        deps = deps + ["tqdm"]
        self._install_dependencies(deps)
        print("Environment ready.")

    def _patch_replace(self, file_name: str, old_text: str, new_text: str) -> None:
        repo_root = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        path = os.path.join(repo_root, file_name)
        
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
    
    def _patch_match(self, file_name: str, pattern: str, replacement: str) -> None:
        """
        Patch a file by replacing text matching a regular expression pattern.
        
        Args:
            file_name: Path to the file relative to the repo root
            pattern: Regular expression pattern to match
            replacement: Replacement text (can include regex groups)
        """
        repo_root = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        path = os.path.join(repo_root, file_name)
        
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

    def _patch(self, patch_text: str) -> None:
        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        
        # Create a temprary file for the patch
        with tempfile.NamedTemporaryFile(mode='w', suffix='.patch', delete=False) as tmp:
            tmp.write(patch_text)
            tmp_path = tmp.name
        
        try:
            print("Applying patch...")
            subprocess.run(["git", "apply", tmp_path], cwd=repo_path, check=True)
        finally:
            os.unlink(tmp_path)

    def _patch_initialize(self, file_names: list[str]) -> None:
        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        
        for file_name in file_names:
            file_path = os.path.join(repo_path, file_name)
            if not os.path.exists(file_path):
                print(f"Warning: File {file_name} does not exist in repository")
                continue
                
            print(f"Resetting {file_name} patches...")
            subprocess.run(["git", "checkout", "--", file_path], cwd=repo_path, check=True)

    def _patch_for_tf2(self, file_names: list[str]):
        """
        Patch files to convert keras imports to tensorflow.keras imports.
        
        Args:
            file_names: List of file paths relative to the repo root
        """
        # Patterns for keras → tf.keras conversions
        keras_patches = [
            (r'^\s*import\s+keras\s*$',                      'import tensorflow.keras as keras'),
            (r'^\s*from\s+keras\.models\b',                  'from tensorflow.keras.models'),
            (r'^\s*from\s+keras\.layers\b',                  'from tensorflow.keras.layers'),
            (r'^\s*from\s+keras\.engine\.topology import Layer\b',        'from tensorflow.keras.layers import Layer'),
            (r'^\s*from\s+keras\s+import\s+backend\b',       'from tensorflow.keras import backend'),
            (r'^\s*from\s+keras\.callbacks\b',               'from tensorflow.keras.callbacks'),
        ]

        for fname in file_names:
            for pattern, replacement in keras_patches:
                self._patch_match(fname, pattern, replacement)

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