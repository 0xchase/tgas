from .base import StaticTGAPlugin, DynamicTGAPlugin

from rmap.core.models import AddressSet
from rmap.core.registry import rmap
from rmap.scan.base import ScanPlugin

from typing import Generic, TypeVar, Optional, Callable, Any, Dict

@rmap(kind="tga", name="6Scan")
class SixScanTga(DynamicTGAPlugin):
    def discover(self,
            seed: AddressSet,
            scanner: ScanPlugin,
            budget: int,
            **kw: Any) -> AddressSet:
        print("Discovering SixScanTga")

