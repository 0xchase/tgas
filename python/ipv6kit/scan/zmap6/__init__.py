# scan/zmap6/__init__.py

# Export the concrete scanner classes for easier access from the scan package
from .icmp_echo import Zmap6ICMPv6EchoScanner
from .tcp_syn import Zmap6TCPSYNScanner

__all__ = [
    "Zmap6ICMPv6EchoScanner",
    "Zmap6TCPSYNScanner",
    # BaseZmap6Scanner is not typically exported as it's an abstract base
    # and resides in .base
]
