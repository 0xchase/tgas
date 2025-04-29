#!/usr/bin/env python3

import sys

from tgas import *
from utils import *

# List of target generation algorithms to use
TGAS = {
    "det":      DET,
    "6Forest":  SixForestTGA,
    "6GCVAE":   SixGcVaeTGA,
    "6Graph":   SixGraphTGA,
    "6Tree":    SixTreeTGA,

    # Requires tensorflow 1.0
    "6VecLM":   SixVecLMTGA,
    "Entropy":  EntropyIp,
    "6GAN":     SixGANTGA,

    # IMPLEMENTS: HMap6, 6Scan, 6Hit, 6Tree, 6Gen
    "6Scan":    SixScanTGA,
}

def main():
    parser = build_parser(TGAS)
    args = parser.parse_args()

    if args.action in ("setup", "train", "generate", "run", "clean"):
        tga_name = getattr(args, "tga_name", None)
        tga = TGAS.get(tga_name)()
        if not tga:
            print(f"Unknown TGA: {tga_name}", file=sys.stderr)
            sys.exit(1)
        
        # dispatch
        if args.action == "setup":
            tga.setup()
        elif args.action == "clean":
            tga.clean()
        else:
            if not os.path.exists(tga.clone_dir):
                print("TGA must be setup before generating or running", file=sys.stderr)
                sys.exit(1)

            if args.action == "generate":
                results = tga.generate(args.count)
                
                # write output to a file if user provided it
                output_path = getattr(args, "output", None)
                if output_path:
                    with open(output_path, "w") as f:
                        for line in results:
                            f.write(line + "\n")
            else:
                if not hasattr(args, 'seeds'):
                    print("Must specify --seeds", file=sys.stderr)
                    sys.exit(1)

                print(f"Loading seeds from {args.seeds}")
                with open(args.seeds, "r") as f:
                    addresses = [line.strip() for line in f if line.strip()]

                    # apply limit if specified
                    if hasattr(args, 'limit') and args.limit is not None:
                        addresses = addresses[:args.limit]

                    if args.action == "train":
                        tga.train(addresses)
                    elif args.action == "run":
                        # run the TGA
                        results = tga.run(addresses, args.count)
                        # handle --output if user provided it
                        output_path = getattr(args, "output", None)
                        if output_path:
                            with open(output_path, "w") as f:
                                for line in results:
                                    f.write(line + "\n")
                        else:
                            # print to stdout
                            for line in results:
                                print(line)
    else:
        parser.print_help()

if __name__ == "__main__":
    main()