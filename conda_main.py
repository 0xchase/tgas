#!/usr/bin/env python3
"""
Python wrapper for IPv6 algorithm pipeline with TGA base class embedding helper routines.
"""

import os
import subprocess

REPO_DIR = "repos"

class TGA:
    """
    Base class for TGAs with embedded helper methods and standardized logging.
    """
    log_dir = None  # class variable; set after LOGDIR is known

    def __init__(self, name, repo_url):
        self.name = name
        self.repo_url = repo_url
        self.clone_dir = os.path.join("repos", name)

    def run_cmd(self, cmd, cwd=None):
        """
        Execute a command (list or string) and log its output.
        """
        shell = isinstance(cmd, str)
        logf = self.get_log_file()
        stdout = open(logf, 'a')
        subprocess.run(cmd, cwd=cwd, shell=shell, stdout=stdout, stderr=subprocess.STDOUT, check=True)
        stdout.close()

    def conda_run(self, script, args=None, cwd=None):
        """Run a script inside this TGA's conda environment."""
        args = args or []
        cmd = ['conda', 'run', '-n', self.name.lower(), script] + args
        self.run_cmd(cmd, cwd)

    def bash_run(self, cmd_str, cwd=None):
        """Run a bash command string within this TGA's context."""
        self.run_cmd(cmd_str, cwd)

    def run_py(self, script, args=None, cwd=None):
        """Run a Python script via the conda env, logging to file."""
        args = args or []
        self.conda_run('python', [script] + args, cwd)

    def setup(self):
        """Optional initialization logic; override in subclass."""
        pass

    def run(self):
        """Main execution logic; override in subclass."""
        pass

# --- TGA subclass for 6GCVAE ---
class SixGCVAE(TGA):
    def __init__(self):
        super().__init__('6GCVAE', "https://github.com/CuiTianyu961030/6GCVAE.git")

    def setup(self):
        dst = os.path.join(self.local_dir, 'data', 'public_datasets')
        os.makedirs(dst, exist_ok=True)
        shutil.copy(SEEDFULL, os.path.join(dst, 'responsive-addresses.txt'))

    def run(self):
        with self.log_context():
            self.setup()
            steps = [
                ('data_process.py', ['--input', SEEDFULL]),
                ('gcnn_vae.py', [self.local_dir]),
                ('generation.py', [self.local_dir])
            ]
            for script, args in steps:
                self.run_py(script, args, cwd=self.local_dir)
        # Collect outputs
        out_dir = os.path.join(RESDIR, self.name)
        os.makedirs(out_dir, exist_ok=True)
        ts = datetime.now().strftime('%Y%m%d_%H%M%S')
        src = os.path.join(self.local_dir, 'data', 'generated_data', '6gcvae_generation.txt')
        dst = os.path.join(out_dir, f"results_{self.name.lower()}_{ts}.txt")
        shutil.copy(src, dst)
