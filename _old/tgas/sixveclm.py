import os
import subprocess
import ipaddress
import random
import tqdm

from .base import StaticTGA, DynamicTGA

class SixVecLMTGA(StaticTGA):
    def setup(self) -> None:
        self.clone("https://github.com/CuiTianyu961030/6VecLM")
        self.install_python("3.7.16")
        self.install_packages(["torch==1.3.1", "torchvision", "torchaudio", "gensim==3.6.0", "scikit-learn", "torchsummary", "matplotlib", "seaborn"])
        self.patch("ipv62vec.py", "min_count=5", "min_count=1")
        self.patch("real_nbatch = math.ceil(len(train_data) / train_batch_size)")

    def train(self, seeds: list[str]) -> None:
        # Write seeds
        seeds_file = os.path.join(self.clone_dir, "data", "public_dataset", "sample_addresses.txt")
        self.write_seeds(seeds, seeds_file, exploded=True)

        if len(seeds) < 20:
            raise Exception("Train requires at least 20 seeds")

        train_size = int(len(seeds) * 4/5)
        eval_size = int(len(seeds) * 1/5)
        batch_size = int(min(100, len(seeds)/5))

        # Patch output file
        os.makedirs(self.train_dir, exist_ok=True)
        self.patch("ipv6_transformer.py", "model_path = \"models/ipv6_transformer_s6_e10_t0015.model\"", "model_path = \"../train/transformer.model\"")

        # Patch data sizes
        self.patch_match("ipv6_transformer.py", r'train_data_size\s*=\s*(\d+)', f'train_data_size = {train_size}')
        self.patch_match("ipv6_transformer.py", r'eval_data_size\s*=\s*(\d+)', f'eval_data_size = {eval_size}')
        self.patch_match("ipv6_transformer.py", r'train_batch_size\s*=\s*(\d+)', f'train_batch_size = {batch_size}')
        self.patch_match("ipv6_transformer.py", r'eval_batch_size\s*=\s*(\d+)', f'eval_batch_size = {batch_size}')

        # Process data
        print("Processing data")
        run = self.cmd([self.python, "data_processing.py"])
        if run.returncode != 0:
            raise RuntimeError(f"data_processing.py failed:\n{run.stderr}")

        # Convert to vectors
        print("Converting to vectors")
        run = self.cmd([self.python, "ipv62vec.py"])
        if run.returncode != 0:
            raise RuntimeError(f"ipv62vec.py failed:\n{run.stderr}")
        
        # Transformer
        run = self.cmd([self.python, "ipv6_transformer.py"])
        if run.returncode != 0:
            raise RuntimeError(f"ipv6_transformer.py failed:\n{run.stderr}")
    
    def generate_batch(self) -> list[str]:
        # Generate a batch
        self.cmd([self.python, "model_load.py"])

        # Collect the generated candidates
        for temperature in [0.020, 0.030, 0.040, 0.050, 0.060, 0.070, 0.080, 0.090, 0.100, 0.200, 0.500]:
            generation_path = os.path.join(self.clone_dir, "data/generation_data/candidate_s6_e1_t" + str(temperature) + ".txt")
            with open(generation_path, "r") as f:
                return f.read().split("\n")

    def generate(self, count: int) -> list[str]:
        model_file = os.path.join(self.train_dir, "transformer.model")
        if not os.path.exists(model_file):
            raise FileNotFoundError("Run train(...) before generate().")
        
        seeds_file = os.path.join(self.clone_dir, "data", "public_dataset", "sample_addresses.txt")
        seed_count = len(open(seeds_file).readlines())
                
        # Patch input model
        self.patch("model_load.py", "model = torch.load(\"models/ipv6_transformer_s6_e10_t0010.model\")", "model = torch.load(\"../train/transformer.model\")")
        self.patch_match("model_load.py", r'train_data_size\s*=\s*(\d+)', f'train_data_size = {seed_count}')

        # Generate unique ips
        unique_ips = set()
        miniters = max(100, count // 100)
        with tqdm.tqdm(total=count, desc=f"Generating unique IPs (each batch may take a while)", miniters=miniters) as pbar:
            while len(unique_ips) < count:
                ips = self.generate_batch()
                print(ips)
                for ip in ips:
                    if ip != "" and ip not in unique_ips:
                        unique_ips.add(ip)
                pbar.update(len(unique_ips))

        return list(unique_ips)[:count]
