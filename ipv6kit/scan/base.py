# ipv6kit/scan/base.py
import datetime
from abc import abstractmethod
from typing import List, Optional, Any
from pydantic import BaseModel, Field

from ipv6kit.core.plugin import BasePlugin
from ipv6kit.core.models import AddressSet

class ScanResult(BaseModel):
    """Data model for the result of a single port/address scan."""
    address: str
    port: int
    protocol: str
    status: str
    timestamp: datetime.datetime = Field(default_factory=datetime.datetime.utcnow)
    banner: Optional[str] = None

class ScanResultSet(BaseModel):
    """Wrapper model for a list of ScanResult, suitable for plugin output."""
    results: List[ScanResult] = Field(default_factory=list)
    scan_name: Optional[str] = None

class ScanPlugin(BasePlugin):
    """scan some addresses"""

    @abstractmethod
    def scan(self, data: AddressSet, **kwargs: Any) -> ScanResultSet:
        pass
