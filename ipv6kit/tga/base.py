# ipv6kit/tga/base.py
from abc import ABC, abstractmethod
from typing import Generic, TypeVar, Optional, Callable, Any, Dict

from ipv6kit.core.plugin import BasePlugin
from ipv6kit.core.models import AddressSet
from ipv6kit.scan.base import ScanPlugin

_ModelT = TypeVar('_ModelT')

class StaticTGAPlugin(BasePlugin, ABC):
    """Base class for TGAs that train a model then generate targets."""
    input_type = AddressSet
    output_type = AddressSet

    @abstractmethod
    def train(self, seed: AddressSet, **kw: Any) -> None:
        ...

    @abstractmethod
    def generate(self, model: _ModelT, num_targets: int, **kw: Any) -> AddressSet:
        ...

class DynamicTGAPlugin(BasePlugin, ABC):
    """Base class for TGAs that interactively explore the address space."""
    input_type = AddressSet
    output_type = AddressSet

    @abstractmethod
    def run(self,
            seed: AddressSet,
            scanner: ScanPlugin,
            budget: int,
            **kw: Any) -> AddressSet:
        pass
