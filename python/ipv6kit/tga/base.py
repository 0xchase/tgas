# ipv6kit/tga/base.py
from typing import Generic, TypeVar, Optional, Callable, Any, Dict

from ipv6kit.core.plugin import BasePlugin
from ipv6kit.core.models import AddressSet
from ipv6kit.scan.base import ScanPlugin

class StaticTGAPlugin(BasePlugin):
    """Static TGA that does not interactively explore the address space."""

    def train(self, seed: AddressSet, **kw: Any) -> None:
        """Train the TGA"""
        pass

    def generate(self, model: int, num_targets: int, **kw: Any) -> AddressSet:
        """Generate a set of targets"""
        pass

class DynamicTGAPlugin(BasePlugin):
    def discover(self,
            seed: AddressSet,
            scanner: ScanPlugin,
            budget: int,
            **kw: Any) -> AddressSet:
        """Discover new targets by scanning the address space"""

        pass
