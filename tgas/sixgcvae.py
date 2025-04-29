import os
import subprocess
import sys
import runpy
import ipaddress
import re

from .base import StaticTGA, DynamicTGA

class SixGcVaeTGA(StaticTGA):
    def setup(self) -> None:
        self.clone("https://github.com/CuiTianyu961030/6GCVAE")
        self.install_python("3.6.15")
        self.install_packages(["scikit-learn", "numpy", "tensorflow-gpu==1.15.5", "keras==2.2.4"])

    def train(self, seeds: list[str]) -> None:
        # Write seeds to a file
        processed_dir = os.path.join(self.clone_dir, 'data', 'processed_data')
        os.makedirs(processed_dir, exist_ok=True)
        data_file = os.path.join(processed_dir, 'data.txt')
        self.write_seeds(seeds, data_file, exploded=True, colan=False)
        
        # Prepare models directory
        models_dir = os.path.join(self.clone_dir, 'models')
        os.makedirs(models_dir, exist_ok=True)

        # Call the existing training script
        training_script = os.path.join(self.clone_dir, 'gcnn_vae.py')
        print("Training the VAE")
        result = self.cmd([self.python, training_script])
        if result.returncode != 0:
            raise RuntimeError(f"Training failed")
        
    def generate(self, count: int) -> list[str]:
        print("Generating addresses")

        gen_py = os.path.join(self.clone_dir, 'generation.py')

        self.patch_match('generation.py', r'generation_number\s*=\s*(\d+)', f'generation_number = {count}')

        result = self.cmd([self.python, gen_py])
        if result.returncode != 0:
            raise RuntimeError(f"Generation failed:\n{result.stderr}")

        # Read back the file it produced
        out_file = os.path.join(self.clone_dir, 'data', 'generated_data', '6gcvae_generation.txt')
        if not os.path.exists(out_file):
            raise FileNotFoundError(f"Expected generation at {out_file}")

        with open(out_file, 'r') as f:
            return [line.strip() for line in f]
