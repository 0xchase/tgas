#!/usr/bin/env python3

import sys

from tgas import *
from utils import *

# Example IPv6 addresses for training
addresses = [
    "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
    "2001:0db8:85a3:0000:0000:8a2e:0370:7335",
    "2001:0db8:85a3:0000:0000:8a2e:0370:7336",
]

# List of target generation algorithms to use
TGAS = {
    "Entropy":  EntropyIp("https://github.com/akamai/entropy-ip.git"),
    "6Tree":    TGA("https://github.com/sixiangdeweicao/6Tree.git"),
    "DET":      TGA("https://github.com/sixiangdeweicao/DET"),
    "6GCVAE":   TGA("https://github.com/CuiTianyu961030/6GCVAE.git"),
    "6VecLM":   TGA("https://github.com/CuiTianyu961030/6VecLM.git"),
    "6GAN":     SixGANTGA("https://github.com/CuiTianyu961030/6GAN.git"),
    "6Graph":   TGA("https://github.com/Lab-ANT/6Graph.git"),
    "6Forest":  SixForestTGA("https://github.com/Lab-ANT/6Forest.git"),
    "6Scan":    TGA("https://github.com/hbn1987/6Scan.git"),
}

def main():
    parser = build_parser(TGAS)
    args = parser.parse_args()

    # figure out which TGA we're using
    if args.action in ("train", "generate", "clean"):
        # e.g. for "train" subcommands, we have "args.tga_name"
        tga_name = getattr(args, "tga_name", None)
        tga = TGAS.get(tga_name)
        if not tga:
            print(f"Unknown TGA: {tga_name}", file=sys.stderr)
            sys.exit(1)
        
        # dispatch
        if args.action == "clean":
            tga.clean()
        else:
            # Initialize before training or generating
            tga.initialize()
            if args.action == "train":
                # Read IPv6 addresses from the input file
                with open(args.input_file, "r") as f:
                    addresses = [line.strip() for line in f if line.strip()]
                tga.train(addresses)
            elif args.action == "generate":
                results = tga.generate(args.count)
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