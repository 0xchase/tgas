import argparse
import ipaddress
import bz2

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
    parser = argparse.ArgumentParser(
        description="Script for testing various TGAs with initialize, train, and generate actions."
    )

    subparsers = parser.add_subparsers(dest="action", required=True, help="Action to perform")

    # -- Action: initialize
    init_parser = subparsers.add_parser("initialize", help="Initialize a TGA.")
    init_sub = init_parser.add_subparsers(dest="tga_name", required=True, help="Which TGA to initialize?")
    for tga_name, tga_cls in tgas.items():
        sp = init_sub.add_parser(tga_name, help=f"Initialize {tga_name}")
        # tga_cls.register_initialize_args(sp)
        # Optionally we can add general arguments, e.g. sp.add_argument(...)
    
    # -- Action: train
    train_parser = subparsers.add_parser("train", help="Train a TGA.")
    train_sub = train_parser.add_subparsers(dest="tga_name", required=True, help="Which TGA to train?")
    for tga_name, tga_cls in tgas.items():
        sp = train_sub.add_parser(tga_name, help=f"Train {tga_name}")
        # tga_cls.register_train_args(sp)
        # Optionally add other shared arguments
    
    # -- Action: generate
    gen_parser = subparsers.add_parser("generate", help="Generate addresses using a TGA.")
    gen_sub = gen_parser.add_subparsers(dest="tga_name", required=True, help="Which TGA to generate with?")
    for tga_name, tga_cls in tgas.items():
        sp = gen_sub.add_parser(tga_name, help=f"Generate with {tga_name}")
        # We can add a shared --output param here:
        sp.add_argument("--output", help="File path to write generated addresses.")
        # Then call the TGA's own generate-arg registration
        # tga_cls.register_generate_args(sp)

    return parser