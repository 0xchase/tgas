import os
import subprocess
import ipaddress
import shutil

from .base import StaticTGA, DynamicTGA

class DET(DynamicTGA):
    def setup(self) -> None:
        self.clone("https://github.com/sixiangdeweicao/DET")
        self.install_python("3.7.16")

    def run(self, addrs: list[str], count: int) -> None:
        output_dir = os.path.join(self.clone_dir, "output")
        seeds_file = os.path.join(output_dir, "seeds.txt")
        self.write_seeds(addrs, seeds_file, exploded=True)

        first = addrs[0]
        # TODO
        source_ip = None

        # Delete old zmap directory
        zmap_dir = os.path.join(output_dir, "zmap")
        if os.path.exists(zmap_dir):
            shutil.rmtree(zmap_dir)
        os.makedirs(zmap_dir)

        cmd = [
            self.env_python,
            "DynamicScan.py",
            "--input", seeds_file,
            "--output", output_dir,
            "--budget", str(count),
            "--IPv6", source_ip,
        ]

        print("Running scan...")
        proc = subprocess.run(
            cmd,
            cwd=repo_path,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )

        if proc.returncode != 0:
            raise RuntimeError(f"DynamicScan.py failed:\n{proc.stderr}")

        discovered = set()
        for fn in os.listdir(zmap_dir):
            if fn.startswith("scan_output_") and fn.endswith(".txt"):
                for line in open(os.path.join(zmap_dir, fn)):
                    addr = line.strip()
                    print(addr)
                    if addr:
                        discovered.add(addr)

        print(f"Discovered {len(discovered)} addresses")

        return list(discovered)
