from .base import StaticTGAPlugin, DynamicTGAPlugin

from rmap.core.models import AddressSet
from rmap.core.registry import rmap
from rmap.scan.base import ScanPlugin

from typing import Generic, TypeVar, Optional, Callable, Any, Dict

@rmap(kind="tga", name="DET")
class DetTga(DynamicTGAPlugin):
    def discover(self,
            seed: AddressSet,
            scanner: ScanPlugin,
            budget: int,
            **kw: Any) -> AddressSet:
        print("Discovering DET")