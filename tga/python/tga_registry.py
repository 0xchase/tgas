"""
TGA Registry - Manages Python-based Target Generation Algorithms

This module provides a registry of Python TGAs and functions to train and generate
from them. It's designed to be called from the tga_runner.py subprocess.
"""

import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Any, Optional
import tempfile
import pickle

# Add the current directory to Python path
current_dir = Path(__file__).parent
sys.path.insert(0, str(current_dir))

try:
    from lstm_tga import LSTMIPv6TGA
except ImportError as e:
    print(f"Warning: Could not import lstm_tga: {e}", file=sys.stderr)

# Registry of available TGAs
registry: Dict[str, Dict[str, Any]] = {}

def register_tga(name: str, description: str, tga_class: type):
    """Register a TGA class in the registry"""
    registry[name] = {
        "description": description,
        "class": tga_class
    }

def list_available_tgas() -> Dict[str, Any]:
    """List all available TGAs"""
    return {
        "tgas": [
            {
                "name": name,
                "description": info["description"]
            }
            for name, info in registry.items()
        ]
    }

def train_tga_from_addresses(tga_name: str, addresses_json: str, **kwargs) -> str:
    """Train a TGA model from a list of addresses"""
    if tga_name not in registry:
        raise ValueError(f"Unknown TGA: {tga_name}")
    
    tga_class = registry[tga_name]["class"]
    addresses = json.loads(addresses_json)
    
    # Convert hex strings to bytes
    address_bytes = [bytes.fromhex(addr) for addr in addresses]
    
    # Train the model
    model = tga_class.train(address_bytes, **kwargs)
    
    # Serialize the model to a temporary file
    with tempfile.NamedTemporaryFile(mode='wb', delete=False, suffix='.pkl') as f:
        pickle.dump(model, f)
        model_path = f.name
    
    return json.dumps({
        "success": True,
        "model_path": model_path,
        "tga_name": tga_name
    })

def generate_from_tga(tga_name: str, model_info_json: str, count: int, unique: bool = False, **kwargs) -> str:
    """Generate addresses using a trained TGA model"""
    model_info = json.loads(model_info_json)
    model_path = model_info["model_path"]
    
    if tga_name not in registry:
        raise ValueError(f"Unknown TGA: {tga_name}")
    
    tga_class = registry[tga_name]["class"]
    
    # Load the model
    with open(model_path, 'rb') as f:
        model = pickle.load(f)
    
    # Generate addresses
    if unique:
        addresses = model.generate_unique(count)
    else:
        addresses = [model.generate() for _ in range(count)]
    
    # Convert bytes to hex strings
    hex_addresses = [addr.hex() for addr in addresses]
    
    return json.dumps({
        "success": True,
        "addresses": hex_addresses
    })

# Auto-register available TGAs
def _auto_register_tgas():
    """Automatically register all available TGAs"""
    try:
        from lstm_tga import LSTMIPv6TGA
        register_tga("lstm_ipv6", "LSTM-based IPv6 address generator", LSTMIPv6TGA)
    except ImportError:
        pass  # LSTM TGA not available

# Initialize the registry
_auto_register_tgas()

if __name__ == "__main__":
    # Test the registry
    print("Available TGAs:")
    for name, info in registry.items():
        print(f"  {name}: {info['description']}") 