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
    "6Forest":  SixForestTGA("https://github.com/Lab-ANT/6Forest.git"),         # DONE
    "6GAN":     SixGANTGA("https://github.com/CuiTianyu961030/6GAN.git"),       # TODO: Requires tensorflow 1.0
    "6GCVAE":   SixGcVaeTGA("https://github.com/CuiTianyu961030/6GCVAE.git"),   # DONE
    "6Graph":   SixGraphTGA("https://github.com/Lab-ANT/6Graph.git"),           # DONE
    "6Scan":    TGA("https://github.com/hbn1987/6Scan.git"),                    # TODO: C++, real-time scan
    "6Tree":    SixTreeTGA("https://github.com/sixiangdeweicao/6Tree.git"),     # TODO: real-time scan
    "6VecLM":   SixVecLMTGA("https://github.com/CuiTianyu961030/6VecLM.git"),           # TODO
    "DET":      TGA("https://github.com/sixiangdeweicao/DET"),                  # TODO
    "Entropy":  EntropyIp("https://github.com/akamai/entropy-ip.git"),          # TODO: Requires tensorflow 1.0
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