# ipv6kit/datasets/base.py
from abc import abstractmethod
from typing import Any

from ipv6kit.core.plugin import BasePlugin
from ipv6kit.core.models import AddressSet

class DatasetPlugin(BasePlugin):
    """
    download a dataset
    """

    @abstractmethod
    def download(self, data: None = None, **kwargs: Any) -> AddressSet:
        pass
