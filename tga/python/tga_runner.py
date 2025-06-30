#!/usr/bin/env python3
"""
TGA Runner - Handles TGA operations via stdin/stdout communication

This script reads JSON commands from stdin and writes JSON responses to stdout.
It's designed to be called as a subprocess from Rust.
"""

import sys
import json
import os
import tempfile
from pathlib import Path

# Add the current directory to Python path so we can import tga_registry
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

try:
    from tga_registry import registry, train_tga_from_addresses, generate_from_tga, list_available_tgas
except ImportError as e:
    print(json.dumps({"error": f"Failed to import tga_registry: {e}"}))
    sys.exit(1)


def handle_list_tgas():
    """List all available TGAs"""
    try:
        result = list_available_tgas()
        print(json.dumps(result))
    except Exception as e:
        print(json.dumps({"error": str(e)}))


def handle_train(command):
    """Train a TGA model"""
    try:
        tga_name = command["tga_name"]
        addresses = command["addresses"]  # List of hex strings
        kwargs = command.get("kwargs", {})
        
        # Train the model - pass hex strings directly
        result = train_tga_from_addresses(tga_name, json.dumps(addresses), **kwargs)
        
        # Parse and return the result
        result_data = json.loads(result)
        print(json.dumps(result_data))
        
    except Exception as e:
        print(json.dumps({"error": str(e)}))


def handle_generate(command):
    """Generate addresses using a trained model"""
    try:
        tga_name = command["tga_name"]
        model_info = command["model_info"]
        count = command["count"]
        unique = command.get("unique", False)
        kwargs = command.get("kwargs", {})
        
        # Generate addresses
        result = generate_from_tga(tga_name, json.dumps(model_info), count, unique, **kwargs)
        
        # Parse and return the result
        result_data = json.loads(result)
        print(json.dumps(result_data))
        
    except Exception as e:
        print(json.dumps({"error": str(e)}))


def main():
    """Main function - read commands from stdin and process them"""
    try:
        # Read the command from stdin
        command_line = sys.stdin.readline().strip()
        if not command_line:
            print(json.dumps({"error": "No command received"}))
            return
        
        command = json.loads(command_line)
        command_type = command.get("command")
        
        if command_type == "list_tgas":
            handle_list_tgas()
        elif command_type == "train":
            handle_train(command)
        elif command_type == "generate":
            handle_generate(command)
        else:
            print(json.dumps({"error": f"Unknown command: {command_type}"}))
            
    except json.JSONDecodeError as e:
        print(json.dumps({"error": f"Invalid JSON: {e}"}))
    except Exception as e:
        print(json.dumps({"error": f"Unexpected error: {e}"}))


if __name__ == "__main__":
    main() 