# ipv6kit/tga/base.py
from typing import Generic, TypeVar, Optional, Callable, Any, Dict

from ipv6kit.core.plugin import BasePlugin
from ipv6kit.core.models import AddressSet
from ipv6kit.scan.base import ScanPlugin

class StaticTGAPlugin(BasePlugin):
    """Base class for TGAs that train a model then generate targets."""

    def train(self, seed: AddressSet, **kw: Any) -> None:
        pass

    def generate(self, model: int, num_targets: int, **kw: Any) -> AddressSet:
        pass

class DynamicTGAPlugin(BasePlugin):
    """Base class for TGAs that interactively explore the address space."""
    input_type = AddressSet
    output_type = AddressSet

    def run(self,
            seed: AddressSet,
            scanner: ScanPlugin,
            budget: int,
            **kw: Any) -> AddressSet:
        pass
