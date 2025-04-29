import argparse
import ipaddress
import bz2

from tgas import StaticTGA, DynamicTGA

def parse_bz2_ipv6_file(filepath: str) -> list[str]:
    addresses = []
    with bz2.open(filepath, mode="rt") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue  # skip blank lines
            
            # Parse the address
            try:
                ipv6_obj = ipaddress.IPv6Address(line)
                addresses.append(ipv6_obj.exploded)  # fully expanded form
            except ipaddress.AddressValueError:
                # If invalid, you can either skip or raise an error.
                # Here we skip it, but you could also do `raise` or log a warning.
                continue
    
    return addresses

def parse_args():
    parser = argparse.ArgumentParser(
        description="Script for testing various TGAs with initialize, train, and generate actions."
    )

    subparsers = parser.add_subparsers(dest="action", required=True, help="Action to perform")

    # Subparser for 'initialize'
    parser_init = subparsers.add_parser("initialize", help="Initialize a specific TGA.")
    parser_init.add_argument(
        "--tga",
        default=None,
        help="Name of the TGA class to initialize (e.g., 'EntropyIPTGA')."
    )

    # Subparser for 'train'
    parser_train = subparsers.add_parser("train", help="Train a specific TGA with some input data.")
    parser_train.add_argument(
        "--tga",
        default=None,
        help="Name of the TGA class to train (e.g., 'EntropyIPTGA')."
    )
    # You might add other arguments here, e.g. input file, seeds, etc.

    # Subparser for 'generate'
    parser_generate = subparsers.add_parser("generate", help="Generate addresses using a specific TGA.")
    parser_generate.add_argument(
        "--tga",
        default=None,
        help="Name of the TGA class to use for generation (e.g., 'EntropyIPTGA')."
    )
    parser_generate.add_argument(
        "--output",
        default=None,
        help="File path to write generated IPs. If omitted, prints to stdout."
    )

    args = parser.parse_args()
    return args

def build_parser(tgas):
    parser = argparse.ArgumentParser(description="Script for running TGAs")

    subparsers = parser.add_subparsers(dest="action", required=True, help="Action to perform")

    # Setup
    setup_parser = subparsers.add_parser("setup", help="Setup a TGA")
    setup_sub = setup_parser.add_subparsers(dest="tga_name", required=True, help="Which TGA to train?")
    for tga_name, tga_cls in tgas.items():
        sp = setup_sub.add_parser(tga_name, help=f"Setup {tga_name}")
        sp.add_argument("--input-file", required=True, help="Path to a file containing IPv6 addresses, one per line.")
        sp.add_argument("--limit", type=int, help="Maximum number of addresses to load for training. If not specified, loads all addresses.")

    # Clean
    clean_parser = subparsers.add_parser("clean", help="Clean a TGA setup")
    clean_sub = clean_parser.add_subparsers(dest="tga_name", required=True, help="Which TGA to clean?")
    for tga_name, tga_cls in tgas.items():
        sp = clean_sub.add_parser(tga_name, help=f"Clean {tga_name}")

    # Train
    train_parser = subparsers.add_parser("train", help="Train a static TGA")
    train_sub = train_parser.add_subparsers(dest="tga_name", required=True, help="Which TGA to train?")
    for tga_name, tga_cls in tgas.items():
        if tga_cls.isinstance(StaticTGA):
            sp = train_sub.add_parser(tga_name, help=f"Train {tga_name}")
            sp.add_argument("--seeds", required=True, help="Path to a file containing IPv6 addresses, one per line.")
            sp.add_argument("--count", type=int, help="Number of addresses to load for training. If not specified, loads all addresses.")
    
    # Generate
    gen_parser = subparsers.add_parser("generate", help="Generate addresses using a TGA")
    gen_sub = gen_parser.add_subparsers(dest="tga_name", required=True, help="Which TGA to generate with?")
    for tga_name, tga_cls in tgas.items():
        if tga_cls.isinstance(StaticTGA):
            sp = gen_sub.add_parser(tga_name, help=f"Generate with {tga_name}")
            sp.add_argument("--count", type=int, required=True, help="Number of addresses to generate")
            sp.add_argument("--output", help="File path to write generated addresses. If omitted, prints to stdout")
    
    # Run
    run_parser = subparsers.add_parser("run", help="Run a dynamic TGA")
    run_sub = run_parser.add_subparsers(dest="tga_name", required=True, help="Which TGA to run?")
    for tga_name, tga_cls in tgas.items():
        if tga_cls.isinstance(DynamicTGA):
            sp = run_sub.add_parser(tga_name, help=f"Run {tga_name}")
            
    return parser