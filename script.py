#!/usr/bin/env python3

import os
import sys
import argparse
import subprocess
import ipaddress
import numpy as np
import bz2

############################################################
# PARENT CLASS: TGA
############################################################

class TGA:
    """
    Base class representing a target generation algorithm.
    """
    def __init__(self, github_url: str, clone_directory: str = "repos"):
        # Use an absolute path for the clone directory
        self.clone_directory = os.path.abspath(clone_directory)
        self.github_url = github_url
        self.repo_name = self._extract_repo_name()

    def _extract_repo_name(self) -> str:
        repo_name = self.github_url.split('/')[-1]
        if repo_name.endswith('.git'):
            repo_name = repo_name[:-4]
        return repo_name

    def _initialize_python(self, python_version: str, deps: list[str]) -> None:
        print(f"Ensuring pyenv has Python {python_version} installed...")
        subprocess.run(["pyenv", "install", "--skip-existing", python_version], check=True)

        # Build the path to the pyenv-managed Python interpreter
        pyenv_root = subprocess.run(["pyenv", "root"], capture_output=True, text=True, check=True).stdout.strip()
        python_executable = os.path.join(pyenv_root, "versions", python_version, "bin", "python")

        # Create venv directory in the repo
        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        env_path = os.path.join(repo_path, "venv")

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
            if not os.path.exists(env_path):
                print(f"Creating Python2.7 virtual environment at {env_path} with virtualenv...")
                subprocess.run([python_executable, "-m", "virtualenv", env_path], check=True)
            else:
                print(f"Virtual environment already exists at {env_path}.")
        else:
            # Use built-in venv
            if not os.path.exists(env_path):
                print(f"Creating virtual environment at {env_path} with {python_executable} -m venv ...")
                subprocess.run([python_executable, "-m", "venv", env_path], check=True)
            else:
                print(f"Virtual environment already exists at {env_path}.")

        # Path to the newly created environment's python
        self.env_python = os.path.join(env_path, "bin", "python")
        if not os.path.exists(self.env_python):
            raise FileNotFoundError(f"Could not find '{self.env_python}' in the virtual environment.")

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

############################################################
# ENTROPYIP IMPLEMENTATION
############################################################

class EntropyIp(TGA):
    def __init__(self, github_url: str, clone_directory: str = "repos"):
        super().__init__(github_url, clone_directory)
        self.env_python = None

    def initialize(self) -> None:
        self._initialize_python("2.7.18", ["toposort", "numpy==1.16.6"])
        # self._initialize_python("2.7.18", ["toposort", "numpy", "matplotlib==2.2.5", "scikits-learn"])
    
    def _write_ips_file(self, ips: list[str], filepath: str) -> None:
        with open(filepath, "w") as f:
            for ip in ips:
                f.write(ip.replace(":", "") + "\n")

    def train(self, ipv6_addresses: list[str]) -> None:
        if not self.env_python:
            raise RuntimeError("Environment not initialized. Call initialize() first.")
        
        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))

        ip_file = os.path.join(repo_path, "ips.txt")
        self._write_ips_file(ipv6_addresses, ip_file)

        output_dir = os.path.join(repo_path, "output")

        print(f"Running Entropy/IP analysis on {ip_file}, output -> {output_dir}")

        repo_path = os.path.join(self.clone_directory, self.repo_name)
        if not os.path.isdir(repo_path):
            raise FileNotFoundError(f"Repo directory not found: {repo_path}")

        # 1. Make sure output_dir exists
        full_output = os.path.abspath(os.path.join(repo_path, output_dir))
        os.makedirs(full_output, exist_ok=True)

        # Confirm the python environment is set
        if not hasattr(self, "env_python") or not os.path.exists(self.env_python):
            raise RuntimeError("env_python is not set or does not exist. "
                               "Make sure _initialize_python was called.")

        # We'll replicate the commands from ALL.sh. 'ALL.sh' does:
        #  cat ip_file | ./a1-segments.py /dev/stdin > $DIR/segments
        #  cat ip_file | ./a2-mining.py /dev/stdin $DIR/segments > $DIR/analysis
        #  cat ip_file | ./a3-encode.py /dev/stdin $DIR/analysis | ./a4-bayes-prepare.sh /dev/stdin > $DIR/bnfinput
        #  ./a5-bayes.sh $DIR/bnfinput > $DIR/cpd
        #  ./b1-webreport.sh $DIR $DIR/segments $DIR/analysis $DIR/cpd

        # Note we must call Python scripts with self.env_python, 
        # but shell scripts are just called normally (assuming +x perms).

        # Script paths (in the same repo folder)
        a1 = os.path.join(repo_path, "a1-segments.py")
        a2 = os.path.join(repo_path, "a2-mining.py")
        a3 = os.path.join(repo_path, "a3-encode.py")
        a4 = os.path.join(repo_path, "a4-bayes-prepare.sh")
        a5 = os.path.join(repo_path, "a5-bayes.sh")
        b1 = os.path.join(repo_path, "b1-webreport.sh")

        # 1) segments
        seg_path = os.path.join(full_output, "segments")
        cmd = (
            f"cat '{ip_file}' | "
            f"'{self.env_python}' '{a1}' /dev/stdin "
            f"> '{seg_path}'"
        )
        subprocess.run(cmd, shell=True, check=True, cwd=repo_path)

        # 2) segment mining
        analysis_path = os.path.join(full_output, "analysis")
        cmd = (
            f"cat '{ip_file}' | "
            f"'{self.env_python}' '{a2}' /dev/stdin '{seg_path}' "
            f"> '{analysis_path}'"
        )
        subprocess.run(cmd, shell=True, check=True, cwd=repo_path)

        # 3) bayes model
        #    cat ip_file | a3-encode.py /dev/stdin analysis | a4-bayes-prepare.sh /dev/stdin > bnfinput
        bnfinput_path = os.path.join(full_output, "bnfinput")
        cmd = (
            f"cat '{ip_file}' | "
            f"'{self.env_python}' '{a3}' /dev/stdin '{analysis_path}' | "
            f"'{a4}' /dev/stdin "
            f"> '{bnfinput_path}'"
        )
        subprocess.run(cmd, shell=True, check=True, cwd=repo_path)

        #    ./a5-bayes.sh bnfinput > cpd
        cpd_path = os.path.join(full_output, "cpd")
        cmd = (
            f"'{a5}' '{bnfinput_path}' "
            f"> '{cpd_path}'"
        )
        subprocess.run(cmd, shell=True, check=True, cwd=repo_path)

        # 4) web report
        #    ./b1-webreport.sh DIR segments analysis cpd
        cmd = (
            f"'{b1}' '{full_output}' '{seg_path}' '{analysis_path}' '{cpd_path}'"
        )
        subprocess.run(cmd, shell=True, check=True, cwd=repo_path)

        print(f"Entropy/IP analysis complete. Results stored in: {full_output}")

    def generate(self, count: int) -> list[str]:
        """
        In the pure delegate approach, you might rely on the cloned repo's code
        for generating addresses. This stub can remain empty or call another script.
        """
        print("No direct generation logic here; relying on cloned repo's code for address generation.")
        return []

############################################################
# SIXFOREST IMPLEMENTATION
############################################################

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

def parse_bz2_ipv6_file(filepath: str) -> list[str]:
    addresses = []
    with bz2.open(filepath, mode="rt") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue  # skip blank lines
            
            # Parse the address
            try:
                ipv6_obj = ipaddress.IPv6Address(line)
                addresses.append(ipv6_obj.exploded)  # fully expanded form
            except ipaddress.AddressValueError:
                # If invalid, you can either skip or raise an error.
                # Here we skip it, but you could also do `raise` or log a warning.
                continue
    
    return addresses

def parse_args():
    parser = argparse.ArgumentParser(
        description="Script for testing various TGAs with initialize, train, and generate actions."
    )

    subparsers = parser.add_subparsers(dest="action", required=True, help="Action to perform")

    # Subparser for 'initialize'
    parser_init = subparsers.add_parser("initialize", help="Initialize a specific TGA.")
    parser_init.add_argument(
        "--tga",
        default=None,
        help="Name of the TGA class to initialize (e.g., 'EntropyIPTGA')."
    )

    # Subparser for 'train'
    parser_train = subparsers.add_parser("train", help="Train a specific TGA with some input data.")
    parser_train.add_argument(
        "--tga",
        default=None,
        help="Name of the TGA class to train (e.g., 'EntropyIPTGA')."
    )
    # You might add other arguments here, e.g. input file, seeds, etc.

    # Subparser for 'generate'
    parser_generate = subparsers.add_parser("generate", help="Generate addresses using a specific TGA.")
    parser_generate.add_argument(
        "--tga",
        default=None,
        help="Name of the TGA class to use for generation (e.g., 'EntropyIPTGA')."
    )
    parser_generate.add_argument(
        "--output",
        default=None,
        help="File path to write generated IPs. If omitted, prints to stdout."
    )

    args = parser.parse_args()
    return args

def build_parser(tgas):
    parser = argparse.ArgumentParser(
        description="Script for testing various TGAs with initialize, train, and generate actions."
    )

    subparsers = parser.add_subparsers(dest="action", required=True, help="Action to perform")

    # -- Action: initialize
    init_parser = subparsers.add_parser("initialize", help="Initialize a TGA.")
    init_sub = init_parser.add_subparsers(dest="tga_name", required=True, help="Which TGA to initialize?")
    for tga_name, tga_cls in tgas.items():
        sp = init_sub.add_parser(tga_name, help=f"Initialize {tga_name}")
        # tga_cls.register_initialize_args(sp)
        # Optionally we can add general arguments, e.g. sp.add_argument(...)
    
    # -- Action: train
    train_parser = subparsers.add_parser("train", help="Train a TGA.")
    train_sub = train_parser.add_subparsers(dest="tga_name", required=True, help="Which TGA to train?")
    for tga_name, tga_cls in tgas.items():
        sp = train_sub.add_parser(tga_name, help=f"Train {tga_name}")
        # tga_cls.register_train_args(sp)
        # Optionally add other shared arguments
    
    # -- Action: generate
    gen_parser = subparsers.add_parser("generate", help="Generate addresses using a TGA.")
    gen_sub = gen_parser.add_subparsers(dest="tga_name", required=True, help="Which TGA to generate with?")
    for tga_name, tga_cls in tgas.items():
        sp = gen_sub.add_parser(tga_name, help=f"Generate with {tga_name}")
        # We can add a shared --output param here:
        sp.add_argument("--output", help="File path to write generated addresses.")
        # Then call the TGA's own generate-arg registration
        # tga_cls.register_generate_args(sp)

    return parser

# Example IPv6 addresses for training
addresses = [
    "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
    "2001:0db8:85a3:0000:0000:8a2e:0370:7335",
    "2001:0db8:85a3:0000:0000:8a2e:0370:7336",
]

# List of target generation algorithms to use
TGAS = {
    "EntropyIP": EntropyIp("https://github.com/akamai/entropy-ip.git"),
    "6Tree": TGA("https://github.com/sixiangdeweicao/6Tree.git"),
    "DET": TGA("https://github.com/sixiangdeweicao/DET"),
    "6GCVAE": TGA("https://github.com/CuiTianyu961030/6GCVAE.git"),
    "6VecLM": TGA("https://github.com/CuiTianyu961030/6VecLM.git"),
    "6GAN": TGA("https://github.com/CuiTianyu961030/6GAN.git"),
    "6Graph": TGA("https://github.com/Lab-ANT/6Graph.git"),
    "6Forest": SixForestTGA("https://github.com/Lab-ANT/6Forest.git"),
    "6Scan": TGA("https://github.com/hbn1987/6Scan.git"),
}

def main():
    parser = build_parser(TGAS)
    args = parser.parse_args()

    # figure out which TGA we're using
    if args.action in ("initialize", "train", "generate"):
        # e.g. for "initialize" subcommands, we have "args.tga_name"
        tga_name = getattr(args, "tga_name", None)
        tga = TGAS.get(tga_name)
        if not tga:
            print(f"Unknown TGA: {tga_name}", file=sys.stderr)
            sys.exit(1)
        
        # dispatch
        if args.action == "initialize":
            tga.initialize()
            # tga.initialize(args)
        elif args.action == "train":
            tga.train()
            # tga.train(args)
        elif args.action == "generate":
            results = tga.generate()
            # results = tga.generate(args)
            # handle --output if user provided it
            output_path = getattr(args, "output", None)
            if output_path:
                with open(output_path, "w") as f:
                    for line in results:
                        f.write(line + "\n")
                print(f"[DEBUG] Wrote {len(results)} addresses to {output_path}")
            else:
                # print to stdout
                for line in results:
                    print(line)
    else:
        parser.print_help()

if __name__ == "__main__":
    main()