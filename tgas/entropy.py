import os
import subprocess
import tqdm

from .base import StaticTGA, DynamicTGA, add_colans

class EntropyIp(StaticTGA):
    def setup(self) -> None:
        self.clone("https://github.com/akamai/entropy-ip")
        self.install_python("2.7.18")
        self.install_packages(["toposort==1.7", "matplotlib", "scikit-learn", "bnfinder"])

    def train(self, seeds: list[str]) -> None:
        # Write seeds
        os.makedirs(self.train_dir, exist_ok=True)
        ip_file = os.path.join(self.train_dir, "seeds.txt")
        self.write_seeds(seeds, ip_file, exploded=True, colan=False)

        # Script paths
        a1 = os.path.join(self.clone_dir, "a1-segments.py")
        a2 = os.path.join(self.clone_dir, "a2-mining.py")
        a3 = os.path.join(self.clone_dir, "a3-encode.py")
        a4 = os.path.join(self.clone_dir, "a4-bayes-prepare.sh")
        a5 = os.path.join(self.clone_dir, "a5-bayes.sh")
        b1 = os.path.join(self.clone_dir, "b1-webreport.sh")

        # segments
        seg_path = os.path.join(self.clone_dir, "segments")
        self.cmd(f"cat '{ip_file}' | '{self.python}' '{a1}' /dev/stdin > '{seg_path}'")

        # segment mining
        analysis_path = os.path.join(self.train_dir, "analysis")
        self.cmd(f"cat '{ip_file}' | '{self.python}' '{a2}' /dev/stdin '{seg_path}' > '{analysis_path}'")

        # bayes model
        bnfinput_path = os.path.join(self.clone_dir, "bnfinput")
        self.cmd(f"cat '{ip_file}' | '{self.python}' '{a3}' /dev/stdin '{analysis_path}' | '{a4}' /dev/stdin > '{bnfinput_path}'")

        # bayes model
        model_path = os.path.join(self.train_dir, "model")
        self.cmd(f"'{a5}' '{bnfinput_path}' > '{model_path}'")

    def generate(self, count: int) -> list[str]:
        # declare files
        model_path = os.path.join(self.train_dir, "model")
        reduced_path = os.path.join(self.train_dir, "reduced")
        analysis_path = os.path.join(self.train_dir, "analysis")
        results_path = os.path.join(self.train_dir, "results")

        unique_ips = set()
        miniters = max(100, count // 100)
        with tqdm.tqdm(total=count, desc="Generating unique IPs", miniters=miniters) as pbar:
            while len(unique_ips) < count:
                # generate targets
                self.cmd(f"{self.python} c1-gen.py {model_path} -n {str(count)} > {reduced_path}")
                self.cmd(f"{self.python} c2-decode.py {reduced_path} {analysis_path} > {results_path}")

                # parse batch
                with open(results_path, "r") as f:
                    lines = f.read().split("\n")
                    for line in lines:
                        if len(unique_ips) >= count:
                            break

                        if len(line) != 32:
                            continue

                        addr = add_colans(line)
                        if not addr in unique_ips:
                            unique_ips.add(addr)
                            pbar.update(1)

        return list(unique_ips)[:count]