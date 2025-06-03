# ipv6kit/scan/__init__.py
from .base import ScannerPlugin, ScanResult, ScanResultSet # Assuming these are defined in scan/base.py

# Import the zmap6 submodule to ensure its __init__.py (and thus plugins) are processed
from . import zmap6

__all__ = [
    "ScannerPlugin",
    "ScanResult",
    "ScanResultSet",
    "zmap6" # Expose the zmap6 module if needed, or just import for side-effects
]
