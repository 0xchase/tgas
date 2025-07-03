from .base import StaticTGAPlugin, DynamicTGAPlugin

from rmap.core.models import AddressSet
from rmap.core.registry import rmap

from typing import Generic, TypeVar, Optional, Callable, Any, Dict

@rmap(kind="tga", name="6Gen")
class SixGenTga(StaticTGAPlugin):
    def train(self, seed: AddressSet, **kw: Any) -> None:
        print("Training SixGanTga")

    def generate(self, model: int, num_targets: int, **kw: Any) -> AddressSet:
        print("Generating SixGanTga")

