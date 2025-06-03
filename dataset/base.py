# ipv6kit/datasets/base.py
from abc import abstractmethod
from typing import Any

from ipv6kit.core.plugin import BasePlugin
from ipv6kit.core.models import AddressSet

class DatasetPlugin(BasePlugin[None, AddressSet]):
    """
    Base class for plugins that generate an initial AddressSet.
    They do not take structured input.
    """
    input_type = None
    output_type = AddressSet

    @abstractmethod
    def run(self, data: None = None, **kwargs: Any) -> AddressSet:
        pass
