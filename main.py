#!/usr/bin/env python3

import sys

from tgas import *
from utils import *

# List of target generation algorithms to use
TGAS = {
    "6Forest":  SixForestTGA("https://github.com/Lab-ANT/6Forest.git"),         # DONE
    "6GCVAE":   SixGcVaeTGA("https://github.com/CuiTianyu961030/6GCVAE.git"),   # DONE
    "6Graph":   SixGraphTGA("https://github.com/Lab-ANT/6Graph.git"),           # DONE
    "6Tree":    SixTreeTGA("https://github.com/sixiangdeweicao/6Tree.git"),     # DONE
    "DET":      SixDETTGA("https://github.com/sixiangdeweicao/DET"),            # DONE

    # Requires tensorflow 1.0
    "6VecLM":   SixVecLMTGA("https://github.com/CuiTianyu961030/6VecLM.git"),   # TODO: 1.0
    "Entropy":  EntropyIp("https://github.com/akamai/entropy-ip.git"),          # TODO: 1.0
    "6GAN":     SixGANTGA("https://github.com/CuiTianyu961030/6GAN.git"),       # TODO: 1.0

    # IMPLEMENTS: HMap6, 6Scan, 6Hit, 6Tree, 6Gen
    "6Scan":    SixScanTGA("https://github.com/hbn1987/6Scan.git"),             # TODO: real-time scan, C++
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
                
                # Apply limit if specified
                if hasattr(args, 'limit') and args.limit is not None:
                    addresses = addresses[:args.limit]
                    print(f"Limited to {len(addresses)} addresses for training")
                
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