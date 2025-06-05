from .base import StaticTGAPlugin, DynamicTGAPlugin

from ipv6kit.core.models import AddressSet
from ipv6kit.core.registry import ipv6kit
from ipv6kit.scan.base import ScanPlugin

from typing import Generic, TypeVar, Optional, Callable, Any, Dict

@ipv6kit(kind="tga", name="DET")
class DetTga(DynamicTGAPlugin):
    def discover(self,
            seed: AddressSet,
            scanner: ScanPlugin,
            budget: int,
            **kw: Any) -> AddressSet:
        print("Discovering DET")