# rmap/datasets/base.py
from abc import abstractmethod
from typing import Any

from rmap.core.plugin import BasePlugin
from rmap.core.models import AddressSet

class DatasetPlugin(BasePlugin):
    """
    download a dataset
    """

    @abstractmethod
    def download(self, data: None = None, **kwargs: Any) -> AddressSet:
        """download a dataset"""
        pass
