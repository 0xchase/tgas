from .base import StaticTGAPlugin, DynamicTGAPlugin

from ipv6kit.core.models import AddressSet
from ipv6kit.core.registry import ipv6kit

from typing import Generic, TypeVar, Optional, Callable, Any, Dict

@ipv6kit(kind="tga", name="6Tree")
class SixTreeTga(StaticTGAPlugin):
    def train(self, seed: AddressSet, **kw: Any) -> None:
        print("Training SixGanTga")

    def generate(self, model: int, num_targets: int, **kw: Any) -> AddressSet:
        print("Generating SixGanTga")

