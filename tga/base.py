# ipv6kit/tga/base.py
from abc import ABC, abstractmethod
from typing import Generic, TypeVar, Optional, Callable, Any, Dict

from ipv6kit.core.plugin import BasePlugin
from ipv6kit.core.models import AddressSet
from ipv6kit.scan.base import ScannerPlugin # Dynamic TGAs need scanner definitions

_ModelT = TypeVar('_ModelT')

ProgressCallback = Callable[[str, Dict[str, Any]], None]

class StaticTGAPlugin(BasePlugin[AddressSet, AddressSet], Generic[_ModelT], ABC):
    """Base class for TGAs that train a model then generate targets."""
    input_type = AddressSet
    output_type = AddressSet

    @abstractmethod
    def train(self, seed: AddressSet, *, progress_cb: Optional[ProgressCallback] = None, **kw: Any) -> _ModelT:
        ...

    @abstractmethod
    def generate_targets(self, model: _ModelT, num_targets: int, *, progress_cb: Optional[ProgressCallback] = None, **kw: Any) -> AddressSet:
        ...

    # The run method orchestrator from the previous version would go here.
    def run(self, data: AddressSet, **kw: Any) -> AddressSet:
        # Placeholder for the orchestration logic (train, generate)
        raise NotImplementedError("Orchestrator run() method must be implemented by concrete StaticTGAPlugin or a new base.")


class DynamicTGAPlugin(BasePlugin[AddressSet, AddressSet], ABC):
    """Base class for TGAs that interactively explore the address space."""
    input_type = AddressSet
    output_type = AddressSet

    @abstractmethod
    def run(self,
            seed: AddressSet,
            *,
            scanner: ScannerPlugin,
            budget: int,
            progress_cb: Optional[ProgressCallback] = None,
            **kw: Any) -> AddressSet:
        pass
