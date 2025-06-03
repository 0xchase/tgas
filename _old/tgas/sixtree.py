import os
import subprocess
import sys
import random
import urllib.request

from .base import TGA
from typing import Optional

TEMP_FILE_NAME = "seeds.txt"

class SixTreeTGA(TGA):
    """
    A single-file approach to 6GAN TGA:
     1) Clone a repo into a subdirectory with its own venv.
     2) Install dependencies in that venv (tensorflow, gensim, scikit-learn, ipaddress).
     3) Defer all logic (train.py, etc.) to the cloned repository.
     4) Uses absolute paths for everything.
    """
    def __init__(self, github_url: str, clone_directory: str = "repos", source_ipv6: Optional[str] = None):
        super().__init__(github_url, clone_directory)
        # Allow override via constructor or LOCAL_IPV6 env var
        self.source_ipv6 = source_ipv6 or os.environ.get("LOCAL_IPV6")
        if not self.source_ipv6:
            # Attempt to detect public IPv6 via external service
            self.source_ipv6 = self._detect_local_ipv6_via_ipify()
        if not self.source_ipv6:
            print("[!] Warning: No source IPv6 provided or detected. Please set LOCAL_IPV6 or pass source_ipv6.")

    def initialize(self) -> None:
        self.clone()
        self._initialize_python("3.9.1", [])

    def train(self, ipv6_addresses: list[str]) -> None:
        if not self.env_python:
            raise RuntimeError("Environment not initialized. Call initialize() first.")
        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        temp_file = os.path.join(repo_path, TEMP_FILE_NAME)
        with open(temp_file, "w") as f:
            for addr in ipv6_addresses:
                f.write(addr + "\n")
        print(f"[+] Wrote {len(ipv6_addresses)} seed addresses to {temp_file}")

    def generate(self, count: int) -> list[str]:
        if not self.env_python:
            raise RuntimeError("Environment not initialized. Call initialize() first.")
        repo_path = os.path.abspath(os.path.join(self.clone_directory, self.repo_name))
        temp_file = os.path.join(repo_path, TEMP_FILE_NAME)
        if not os.path.exists(temp_file):
            raise FileNotFoundError(f"Seed file {temp_file} not found, run train() first")

        if not self.source_ipv6:
            self.source_ipv6 = self._detect_local_ipv6_via_ipify()
        if not self.source_ipv6:
            raise RuntimeError("No source IPv6 provided or detected. Set LOCAL_IPV6 or pass source_ipv6.")

        # Prepare output directories for scan results
        output_dir = os.path.join(repo_path, "scan_output")
        zmap_dir = os.path.join(output_dir, "zmap")
        os.makedirs(zmap_dir, exist_ok=True)

        budget = count
        dyn_scan = os.path.join(repo_path, "DynamicScan.py")
        cmd = [
            self.env_python,
            dyn_scan,
            "--input", temp_file,
            "--budget", str(budget),
            "--IPv6", self.source_ipv6,
            "--output", output_dir
        ]

        print(f"[+] Running dynamic scan: {' '.join(cmd)}")
        subprocess.run(cmd, check=True)

        # Read the targets file produced by DynamicScan
        target_file = os.path.join(output_dir, f"6Tree.target{budget}")
        if not os.path.exists(target_file):
            raise FileNotFoundError(f"Expected target file {target_file} not found")

        with open(target_file) as f:
            all_targets = [line.strip() for line in f if line.strip()]
        if len(all_targets) < count:
            raise ValueError(f"Requested {count} addresses, but only {len(all_targets)} targets available")

        subset = random.sample(all_targets, count)
        print(f"[+] Generated {len(subset)} target addresses from scan output")
        return subset

    @staticmethod
    def _detect_local_ipv6_via_ipify(timeout: float = 5.0) -> Optional[str]:
        """
        Queries a public IPv6 echo service (api64.ipify.org) to determine the host's public IPv6.
        """
        try:
            with urllib.request.urlopen("https://api64.ipify.org?format=text", timeout=timeout) as resp:
                ip = resp.read().decode().strip()
                if ":" in ip:
                    return ip
        except Exception:
            return None
        return None
