# cli/main.py
import argparse
import inspect
import json
import logging
import pathlib
import sys
from typing import Any, Dict, Optional, List, Union, get_args, get_origin

from pydantic import BaseModel # For type checking and parsing

# Assuming ipv6kit is installed or in PYTHONPATH
from ipv6kit.core.registry import PLUGINS, PLUGIN_KINDS
from ipv6kit.core.plugin import BasePlugin
from ipv6kit.cli.runners.local_runner import LocalRunner # Default runner

# --- Global Variables ---
logger = logging.getLogger("ipv6kit.cli")

# --- Utility Functions ---
def setup_cli_logging(log_level_str: str = "INFO", log_file: Optional[pathlib.Path] = None):
    """Configures basic logging for the CLI."""
    numeric_level = getattr(logging, log_level_str.upper(), None)
    if not isinstance(numeric_level, int):
        raise ValueError(f"Invalid log level: {log_level_str}")

    handlers: List[logging.Handler] = [logging.StreamHandler(sys.stdout)]
    if log_file:
        try:
            file_handler = logging.FileHandler(log_file, mode='a')
            handlers.append(file_handler)
        except Exception as e:
            # Use root logger here as plugin loggers might not be fully set up or CLI logger is what we have
            logging.error(f"Failed to set up log file at {log_file}: {e}")


    logging.basicConfig(
        level=numeric_level,
        format="%(asctime)s - %(name)s [%(levelname)s] - %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
        handlers=handlers
    )
    logger.info(f"CLI logging initialized at level {log_level_str.upper()}.")
    if log_file:
        logger.info(f"Logging additionally to file: {log_file}")


def load_plugin_modules():
    """
    Imports plugin modules to trigger their registration.
    In a production system, this might use entry_points or a plugin discovery mechanism.
    """
    logger.debug("Attempting to load plugin modules...")
    try:
        import ipv6kit.dataset.base
        logger.debug("Loaded ipv6kit.dataset.base")
    except ImportError as e:
        logger.warning(f"Could not load dataset plugins: {e}")

    try:
        import ipv6kit.scan.base
        import ipv6kit.scan.zmap6 # Specifically to load Zmap6 scanners
        logger.debug("Loaded ipv6kit.scan.base and ipv6kit.scan.zmap6")
    except ImportError as e:
        logger.warning(f"Could not load scan plugins: {e}")

    try:
        import ipv6kit.tga.base
        logger.debug("Loaded ipv6kit.tga.base")
    except ImportError as e:
        logger.warning(f"Could not load tga plugins: {e}")

    try:
        import ipv6kit.analyze.base
        logger.debug("Loaded ipv6kit.analyze.base")
    except ImportError as e:
        logger.warning(f"Could not load analyze plugins: {e}")
    logger.debug("Plugin module loading attempt complete.")


def get_underlying_type(annotation: Any) -> Any:
    """Gets the underlying type from Optional or Union types for argparse."""
    origin = get_origin(annotation)
    if origin is Union: # Handles Optional[X], which is Union[X, None]
        args = get_args(annotation)
        non_none_args = [arg for arg in args if arg is not type(None)]
        if len(non_none_args) == 1:
            return non_none_args[0]
    elif origin is list or origin is List: # For List[str] etc.
        list_arg = get_args(annotation)
        if list_arg: return list_arg[0] # Return the inner type for nargs
    return annotation


def add_plugin_arguments(parser: argparse.ArgumentParser, plugin_cls: Type[BasePlugin]):
    """
    Inspects a plugin's __init__ method and adds corresponding CLI arguments.
    """
    sig = inspect.signature(plugin_cls.__init__)
    for name, param in sig.parameters.items():
        if name in ("self", "args", "kwargs", "progress_bars_enabled"): # Skip standard/handled params
            continue

        arg_name = f"--{name.replace('_', '-')}"
        arg_kwargs: Dict[str, Any] = {}

        underlying_type = get_underlying_type(param.annotation)

        if param.annotation == bool or underlying_type == bool:
            if param.default is True:
                arg_name = f"--no-{name.replace('_', '-')}"
                arg_kwargs["action"] = "store_false"
                arg_kwargs["dest"] = name # Ensure args.name matches param name
            else: # Default False or no default (assume False for bool flags)
                arg_kwargs["action"] = "store_true"
        elif get_origin(param.annotation) is list or get_origin(param.annotation) is List:
            arg_kwargs["type"] = underlying_type if underlying_type not in (Any, inspect.Parameter.empty) else str
            arg_kwargs["nargs"] = "+"
        elif underlying_type not in (Any, inspect.Parameter.empty):
            arg_kwargs["type"] = underlying_type
        
        if param.default is not inspect.Parameter.empty:
            if not (param.annotation == bool or underlying_type == bool): # Default is implicit for store_true/false
                 arg_kwargs["default"] = param.default
        else: # No default value
            if not (param.annotation == bool or underlying_type == bool): # Boolean flags are implicitly optional
                arg_kwargs["required"] = True
        
        # Try to generate help string (can be improved with docstrings)
        help_str = f"{name} ({param.annotation})"
        if "default" in arg_kwargs :
             help_str += f" (default: {arg_kwargs['default']})"
        arg_kwargs["help"] = help_str

        parser.add_argument(arg_name, **arg_kwargs)

    # Common arguments for all executable plugins
    parser.add_argument(
        "--input-file", type=pathlib.Path,
        help="Path to JSON input file for the plugin (if applicable)."
    )
    parser.add_argument(
        "--output-file", type=pathlib.Path,
        help="Path to save JSON output from the plugin."
    )
    parser.add_argument(
        "--progress-bars", action="store_true",
        help="Enable progress bars for interactive output (requires tqdm)."
    )


# --- Command Handler Functions ---
def handle_not_implemented(args: argparse.Namespace):
    logger.error(f"Command '{args.command}' (plugin: {args.plugin_name if hasattr(args, 'plugin_name') else 'N/A'}) is not yet implemented.")
    raise NotImplementedError(f"Command '{args.command}' is not yet implemented.")


def handle_plugin_execution(args: argparse.Namespace):
    """Handles the execution of a loaded plugin."""
    plugin_cls: Type[BasePlugin] = args.plugin_cls
    logger.info(f"Executing plugin: {plugin_cls.name} (version {plugin_cls.version})")

    init_params = inspect.signature(plugin_cls.__init__).parameters
    plugin_init_args: Dict[str, Any] = {}

    for name in init_params:
        if name in ("self", "args", "kwargs"):
            continue
        if hasattr(args, name) and getattr(args, name) is not None:
            plugin_init_args[name] = getattr(args, name)
        # Defaults are handled by plugin's __init__ if not present in args

    plugin_init_args["progress_bars_enabled"] = args.progress_bars

    input_data: Optional[BaseModel] = None
    if plugin_cls.input_type and args.input_file:
        if not args.input_file.exists():
            logger.error(f"Input file not found: {args.input_file}")
            sys.exit(1)
        try:
            logger.info(f"Loading input data from: {args.input_file}")
            input_data = plugin_cls.input_type.model_validate_json(args.input_file.read_text())
        except Exception as e:
            logger.error(f"Failed to parse input file {args.input_file} as {plugin_cls.input_type.__name__}: {e}", exc_info=True)
            sys.exit(1)
    elif plugin_cls.input_type and not args.input_file:
         # Check if input is actually required (vs. optional) by plugin design.
         # For now, if input_type is defined, we assume input_file is the primary way.
         # Some plugins (like datasets) have input_type = None.
        logger.warning(f"Plugin {plugin_cls.name} expects input type {plugin_cls.input_type.__name__}, but --input-file was not provided.")
        # Decide if this is an error or if plugin can proceed. For now, proceed with None.


    # Prepare for runner - currently, all args go to __init__
    # The runner's execute method separates init_args from run_method_args.
    # For now, our plugins put most config in __init__.
    run_method_args: Dict[str, Any] = {} # Placeholder for future run-specific CLI args

    runner = LocalRunner() # Using LocalRunner by default
    try:
        logger.info(f"Instantiating plugin {plugin_cls.name} with args: {plugin_init_args}")
        # The LocalRunner will instantiate the plugin.
        # We pass the class and its init args.
        output_data = runner.execute(
            plugin_cls=plugin_cls,
            plugin_init_args=plugin_init_args,
            run_method_args=run_method_args,
            input_data=input_data
            # No explicit progress_cb from CLI to runner for now;
            # plugin's internal progress bars are controlled by 'progress_bars_enabled'
        )
        logger.info(f"Plugin {plugin_cls.name} execution finished.")

        if args.output_file:
            logger.info(f"Saving output to: {args.output_file}")
            try:
                args.output_file.parent.mkdir(parents=True, exist_ok=True)
                args.output_file.write_text(output_data.model_dump_json(indent=2))
                logger.info(f"Output successfully saved to {args.output_file}")
            except Exception as e:
                logger.error(f"Failed to save output to {args.output_file}: {e}", exc_info=True)
                # Optionally, print to stdout as a fallback
                print("\n--- Plugin Output (JSON) ---")
                print(output_data.model_dump_json(indent=2))
        else:
            logger.info("Printing output to stdout (JSON).")
            print("\n--- Plugin Output (JSON) ---")
            print(output_data.model_dump_json(indent=2))

    except Exception as e:
        logger.critical(f"An error occurred during plugin {plugin_cls.name} execution: {e}", exc_info=True)
        sys.exit(1)
    finally:
        runner.close()


# --- Main CLI Parsing ---
def main():
    # Load plugins first so they are available for parser generation
    # Basic logging for plugin loading itself
    pre_setup_logging_level = "INFO"
    if "-v" in sys.argv or "--verbose" in sys.argv:
        pre_setup_logging_level = "DEBUG"
    # Call basicConfig directly before full setup for early messages
    logging.basicConfig(level=pre_setup_logging_level.upper(), format="%(asctime)s - %(name)s [%(levelname)s] - %(message)s")

    load_plugin_modules()

    parser = argparse.ArgumentParser(
        description="ipv6kit: A Toolkit for IPv6 Research and Analysis.",
        formatter_class=argparse.RawTextHelpFormatter
    )
    parser.add_argument(
        "-v", "--verbose", action="store_const", const="DEBUG", dest="log_level",
        help="Enable verbose (DEBUG level) logging to stdout."
    )
    parser.add_argument(
        "--log-file", type=pathlib.Path,
        help="Path to a file for logging (appends)."
    )
    parser.set_defaults(log_level="INFO") # Default log level if -v not used

    # Top-level command subparsers
    command_subparsers = parser.add_subparsers(dest="command", title="Commands", required=True)

    # --- Create subparsers for each plugin kind ---
    for kind in PLUGIN_KINDS:
        kind_parser = command_subparsers.add_parser(kind, help=f"Access {kind} plugins.")
        kind_plugin_subparsers = kind_parser.add_subparsers(dest="plugin_name", title=f"Available {kind} plugins", required=True)

        if not PLUGINS[kind]:
            logger.debug(f"No plugins found for kind: {kind}")
            # Add a dummy parser to indicate no plugins if desired, or let it be empty
            # no_plugins_parser = kind_plugin_subparsers.add_parser("none", help=f"No {kind} plugins currently registered.")
            # no_plugins_parser.set_defaults(handler_func=lambda args: print(f"No {kind} plugins available."))
            continue


        for plugin_name, plugin_cls in PLUGINS[kind].items():
            plugin_parser = kind_plugin_subparsers.add_parser(
                plugin_name,
                help=plugin_cls.description or f"Run the {plugin_name} {kind} plugin.",
                description=plugin_cls.description or f"Run the {plugin_name} {kind} plugin. Version: {plugin_cls.version}"
            )
            plugin_parser.set_defaults(plugin_cls=plugin_cls)
            add_plugin_arguments(plugin_parser, plugin_cls)

            # Set handler function based on kind
            if kind == "scan":
                plugin_parser.set_defaults(handler_func=handle_plugin_execution)
            else: # dataset, tga, analyze
                plugin_parser.set_defaults(handler_func=handle_not_implemented)

    if len(sys.argv) == 1:
        parser.print_help(sys.stderr)
        sys.exit(1)
        
    args = parser.parse_args()

    # Setup logging based on parsed args (after parser is defined)
    setup_cli_logging(args.log_level, args.log_file)


    # Execute the handler function determined by subparsers
    if hasattr(args, "handler_func"):
        args.handler_func(args)
    else:
        # Should not happen if subparsers are marked as required
        logger.error("No command or plugin specified.")
        parser.print_help()
        sys.exit(1)

if __name__ == "__main__":
    main()
