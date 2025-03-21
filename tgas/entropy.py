import os
import subprocess

from .base import TGA

class EntropyIp(TGA):
    def __init__(self, github_url: str, clone_directory: str = "repos"):
        super().__init__(github_url, clone_directory)
        self.env_python = None

    def initialize(self) -> None:
        self._initialize_python("2.7.18", ["toposort", "numpy==1.16.6"])
    
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