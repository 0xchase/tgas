import os
import subprocess

from .base import StaticTGA, DynamicTGA

class EntropyIp(StaticTGA):
    def setup(self) -> None:
        self.clone("https://github.com/akamai/entropy-ip")
        self.install_python("2.7.18")
        self.install_packages(["toposort==1.7", "matplotlib", "scikit-learn", "bnfinder"])

    def train(self, seeds: list[str]) -> None:
        print(f"Writing {len(seeds)} seeds to train directory")
        os.makedirs(self.train_dir, exist_ok=True)
        ip_file = os.path.join(self.train_dir, "seeds.txt")
        self.write_seeds(seeds, ip_file, colan=False)

        if not hasattr(self, "python") or not os.path.exists(self.python):
            raise RuntimeError("python is not set up.")

        # the commands from ALL.sh. 'ALL.sh' does:
        #  cat ip_file | ./a1-segments.py /dev/stdin > $DIR/segments
        #  cat ip_file | ./a2-mining.py /dev/stdin $DIR/segments > $DIR/analysis
        #  cat ip_file | ./a3-encode.py /dev/stdin $DIR/analysis | ./a4-bayes-prepare.sh /dev/stdin > $DIR/bnfinput
        #  ./a5-bayes.sh $DIR/bnfinput > $DIR/cpd
        #  ./b1-webreport.sh $DIR $DIR/segments $DIR/analysis $DIR/cpd

        # Script paths
        a1 = os.path.join(self.clone_dir, "a1-segments.py")
        a2 = os.path.join(self.clone_dir, "a2-mining.py")
        a3 = os.path.join(self.clone_dir, "a3-encode.py")
        a4 = os.path.join(self.clone_dir, "a4-bayes-prepare.sh")
        a5 = os.path.join(self.clone_dir, "a5-bayes.sh")
        b1 = os.path.join(self.clone_dir, "b1-webreport.sh")

        # segments
        print("Generating segments")
        seg_path = os.path.join(self.clone_dir, "segments")
        self.train_cmd(f"cat '{ip_file}' | '{self.python}' '{a1}' /dev/stdin > '{seg_path}'")

        # segment mining
        print("Mining segments")
        analysis_path = os.path.join(self.clone_dir, "analysis")
        self.train_cmd(f"cat '{ip_file}' | '{self.python}' '{a2}' /dev/stdin '{seg_path}' > '{analysis_path}'")

        # bayes model
        #    cat ip_file | a3-encode.py /dev/stdin analysis | a4-bayes-prepare.sh /dev/stdin > bnfinput
        print("Bayes model")
        bnfinput_path = os.path.join(self.clone_dir, "bnfinput")
        self.train_cmd(f"cat '{ip_file}' | '{self.python}' '{a3}' /dev/stdin '{analysis_path}' | '{a4}' /dev/stdin > '{bnfinput_path}'")

        #    ./a5-bayes.sh bnfinput > cpd
        print("Bayes model 2")
        cpd_path = os.path.join(self.clone_dir, "cpd")
        cmd = f"'{a5}' '{bnfinput_path}' > '{cpd_path}'"
        print(cmd)
        self.train_cmd(cmd)

        # web report
        #    ./b1-webreport.sh DIR segments analysis cpd
        #cmd = (
        #    f"'{b1}' '{full_output}' '{seg_path}' '{analysis_path}' '{cpd_path}'"
        #)
        #subprocess.run(cmd, shell=True, check=True, cwd=repo_path)

        #print(f"Entropy/IP analysis complete. Results stored in: {full_output}")

    def generate(self, count: int) -> list[str]:
        """
        In the pure delegate approach, you might rely on the cloned repo's code
        for generating addresses. This stub can remain empty or call another script.
        """
        print("No direct generation logic here; relying on cloned repo's code for address generation.")
        return []