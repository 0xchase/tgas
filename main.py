#!/usr/bin/env python3

import sys

from tgas import *
from utils import *

TGAS = {
    "det":      DET,            # TODO (dynamic)
    "entropy":  EntropyIp,      # PARTIAL (hangs)
    "6Forest":  SixForestTGA,   # DONE
    "6GCVAE":   SixGcVaeTGA,    # DONE
    "6Graph":   SixGraphTGA,    # DONE
    "6Tree":    SixTreeTGA,     # TODO (dynamic)
    "6VecLM":   SixVecLMTGA,    # PARTIAL (error)
    "6GAN":     SixGANTGA,      # PARTIAL (hangs)

    # IMPLEMENTS: HMap6, 6Scan, 6Hit, 6Tree, 6Gen
    "6Scan":    SixScanTGA,     # TODO (dynamic)
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
            tga.log = os.path.join(tga.setup_dir, "setup.log")
            tga.setup()
        elif args.action == "clean":
            tga.clean()
        else:
            if not os.path.exists(tga.clone_dir):
                tga.log = os.path.join(tga.setup_dir, "setup.log")
                tga.setup()

            if args.action == "generate":
                tga.log = os.path.join(tga.setup_dir, "generate.log")
                results = tga.generate(args.count)
                
                # write output to a file if user provided it
                output_path = getattr(args, "output", None)
                if output_path:
                    with open(output_path, "w") as f:
                        for line in results:
                            f.write(line + "\n")
                else:
                            for line in results:
                                print(line)
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
                        tga.log = os.path.join(tga.setup_dir, "train.log")
                        tga.train(addresses)
                    elif args.action == "run":
                        # run the TGA
                        tga.log = os.path.join(tga.setup_dir, "run.log")
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