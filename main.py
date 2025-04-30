#!/usr/bin/env python3

import sys

from tgas import *
from utils import *

TGAS = {
    "6Forest":  SixForestTGA,   # DONE
    "6Graph":   SixGraphTGA,    # DONE
    "6VecLM":   SixVecLMTGA,    # DONE
    "6GAN":     SixGANTGA,      # PARTIAL (hangs)

    "6Gen":     SixGenTGA,      # DONE (linux-only)
    "6GCVAE":   SixGcVaeTGA,    # DONE (linux-only)
    "entropy":  EntropyIp,      # DONE (linux-only)

    "det":      DET,            # TODO (dynamic)
    "6Tree":    SixTreeTGA,     # TODO (dynamic)

    # IMPLEMENTS: HMap6, 6Scan, 6Hit, 6Tree, 6Gen
    "6Scan":    SixScanTGA,     # TODO (dynamic)
}

def write_lines(lines, output=None):
    if output:
        with open(output, 'w') as f:
            for line in lines:
                f.write(line + "\n")
    else:
        print(*lines, sep="\n")

def ensure_setup(tga, log_filename):
    tga.log = os.path.join(tga.setup_dir, log_filename)
    tga.setup()

def main():
    parser = build_parser(TGAS)
    args = parser.parse_args()
    action = args.action
    tga_name = getattr(args, "tga_name", None)

    # Validate action
    if action not in {"setup", "train", "generate", "run", "clean"}:
        parser.print_help()
        return

    # Look up TGA class
    tga_cls = TGAS.get(tga_name)
    if not tga_cls:
        print(f"Unknown TGA: {tga_name}", file=sys.stderr)
        sys.exit(1)

    tga = tga_cls()

    # Setup and clean are standalone
    if action == "setup":
        ensure_setup(tga, "setup.log")
        return
    if action == "clean":
        tga.clean()
        return

    # Ensure repository is cloned for other actions
    if not os.path.exists(tga.clone_dir):
        ensure_setup(tga, "setup.log")

    # Generate action
    if action == "generate":
        tga.log = os.path.join(tga.setup_dir, "generate.log")
        results = tga.generate(args.count)
        write_lines(results, getattr(args, "output", None))
        return

    # Train and run actions require seeds
    seeds = getattr(args, "seeds", None)
    if not seeds:
        print("Must specify --seeds", file=sys.stderr)
        sys.exit(1)

    # Load seed addresses
    with open(seeds) as f:
        addresses = [line.strip() for line in f if line.strip()]

    # Apply limit if provided
    limit = getattr(args, "limit", None)
    if limit is not None:
        addresses = addresses[:limit]

    if action == "train":
        tga.log = os.path.join(tga.setup_dir, "train.log")
        tga.train(addresses)
    else:
        tga.log = os.path.join(tga.setup_dir, "run.log")
        results = tga.run(addresses, args.count)
        write_lines(results, getattr(args, "output", None))

if __name__ == "__main__":
    main()
