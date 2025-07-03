import sys
import pathlib
import logging

import argparse # For pre-parsing global flags
from typing import Optional, List, Any, Dict, Type, Generic # For type hints
import inspect
from pydantic import BaseModel # For serializing actual results

#from rich_argparse import RichHelpFormatter

from rmap.core.registry import get_all_plugins
from rmap.core.plugin import BasePlugin # Needed for isinstance checks

logger = logging.getLogger(__name__) # Logger for these utils

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
    import rmap.analyze
    import rmap.dataset
    import rmap.scan
    import rmap.tga

    all_plugins = get_all_plugins()

    # Get all base classes
    base_classes = set()
    for plugin in all_plugins:
        if issubclass(plugin, BasePlugin):
            hierarchy = inspect.getmro(plugin)
            base_class = hierarchy[hierarchy.index(BasePlugin) - 1]
            base_classes.add(base_class)

    parser = argparse.ArgumentParser(
        prog=sys.argv[0],
        description="A simple example of argparse",
        #epilog="And this is the epilog, also supporting multiple lines and Rich formatting.",
        #formatter_class=RichHelpFormatter,
    )

    # add a plugin argument to the parser
    subparsers = parser.add_subparsers(help='command help', dest='command_parsers')

    for base_cls in base_classes:
        # Collect all the commands for that plugin  kind
        for name, member in inspect.getmembers_static(base_cls):
            if not name.startswith('_') and callable(member):
                # create a subparser for the command
                command_parser = subparsers.add_parser(name, help=inspect.getdoc(member))
                spec = inspect.getfullargspec(member)
                # iterate over all the arguments
                for arg in spec.args:
                    if arg == 'self': continue
                    annotation = spec.annotations[arg]

                    if annotation == None:
                        command_parser.add_argument(f"--{arg}", type=annotation, help=f"Help for {arg}")
                    elif inspect.isclass(annotation) and issubclass(annotation, BasePlugin):
                        plugin_names = [plugin.name for plugin in all_plugins if issubclass(plugin, annotation)]
                        command_parser.add_argument(f"--{arg}", choices=plugin_names, help=f"Help for {arg}")
                    elif issubclass(annotation, BaseModel):
                        command_parser.add_argument(f"--{arg}", type=annotation, help=f"Help for {arg}")
                    else:
                        command_parser.add_argument(f"--{arg}", type=annotation, help=f"Help for {arg}")

                plugin_names = [plugin.name for plugin in all_plugins if issubclass(plugin, base_cls)]
                command_parser.add_argument("-p", "--plugin", choices=plugin_names, help="Plugin to use for this command")

    # Add arguments
    parser.add_argument("-v", "--verbose", help="Increase output verbosity", action="store_true")
    parser.add_argument("-l", "--log-file", help="Log file", default="output.log")

    # Parse the arguments
    args = parser.parse_args()

    setup_cli_logging_fire(
        verbose=args.verbose,
        log_file_path_str=args.log_file
    )

if __name__ == '__main__':
    main()