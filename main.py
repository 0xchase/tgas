# /home/crkanip/ipv6kit/main.py
# (initial imports: sys, pathlib, logging, fire, argparse, typing, etc. remain the same)
import sys
import pathlib
import logging

import argparse # For pre-parsing global flags
from typing import Optional, List, Any, Dict, Type, Generic # For type hints
import inspect
import textwrap
import json
from pydantic import BaseModel # For serializing actual results

from ipv6kit.core.registry import get_all_plugins
from ipv6kit.core.plugin import BasePlugin # Needed for isinstance checks

from ipv6kit.analyze.base import AnalyzePlugin
from ipv6kit.dataset.base import DatasetPlugin
from ipv6kit.scan.base import ScanPlugin
from ipv6kit.tga.base import *

# --- Start of sys.path modification (as before) ---
script_file_path = pathlib.Path(__file__).resolve()
ipv6kit_package_dir = script_file_path.parent
project_root_dir = ipv6kit_package_dir.parent
if str(project_root_dir) not in sys.path:
    sys.path.insert(0, str(project_root_dir))
# --- End of sys.path modification ---

logger = logging.getLogger("ipv6kit-cli") # Your CLI logger

import inspect
import textwrap
import logging
from typing import Any, List, Dict, Optional, Callable
from dataclasses import dataclass, field

# Assuming BasePlugin is accessible for type checking if needed, e.g.,
# from ipv6kit.core.plugin import BasePlugin

logger = logging.getLogger(__name__) # Logger for these utils
LINE_WIDTH = 120
DEFAULT_INDENT = "  " # Two spaces for indentation

def setup_cli_logging_fire(verbose: bool = False, log_file_path_str: Optional[str] = None):
    log_level = logging.DEBUG if verbose else logging.INFO
    handlers: List[logging.Handler] = [logging.StreamHandler(sys.stdout)]
    if log_file_path_str:
        try:
            log_file_path = pathlib.Path(log_file_path_str)
            log_file_path.parent.mkdir(parents=True, exist_ok=True)
            file_handler = logging.FileHandler(log_file_path, mode='a')
            handlers.append(file_handler)
        except Exception as e:
            print(f"Error: Failed to set up log file at '{log_file_path_str}': {e}", file=sys.stderr)

    logging.basicConfig(
        level=log_level,
        format="%(name)s [%(levelname)s] - %(message)s",
        handlers=handlers,
        force=True
    )

def main():
    import ipv6kit.analyze
    import ipv6kit.dataset
    import ipv6kit.scan
    # import ipv6kit.tga

    all_plugins = get_all_plugins()

    parser = argparse.ArgumentParser(description="A simple example of argparse")
    subparsers = parser.add_subparsers(help='command help', dest='command_parsers')

    for (kind, plugins) in all_plugins.items():
        # Map command to list of plugins
        commands = {}

        # Collect all the commands for that plugin  kind
        for (kind, plugin_cls) in plugins.items():
            print(kind, plugin_cls)

            for name, member in inspect.getmembers(plugin_cls):
                if not name.startswith('_') and callable(member):
                    if name not in commands:
                        commands[name] = []
                    commands[name].append(plugin_cls)
        
        # Create a parser for each command
        for command, plugins in commands.items():
            docstring = command + " help" or f"{command} help"
            command_parser = subparsers.add_parser(command, help = f"{docstring}")

            # collect all the plugin annotated names
            plugin_names = [plugin.__name__ for plugin in plugins]

            # Optional argument to specify a plugin or plugins only from the list of plugins
            command_parser.add_argument("-p", "--plugin", help="Available plugins: " + ", ".join(plugin_names), action="append")

    # Add arguments
    parser.add_argument("-v", "--verbose", help="Increase output verbosity", action="store_true")
    parser.add_argument("-l", "--log-file", help="Log file", default="ipv6kit.log")

    # Parse the arguments
    args = parser.parse_args()

    setup_cli_logging_fire(
        verbose=args.verbose,
        log_file_path_str=args.log_file
    )

if __name__ == '__main__':
    main()