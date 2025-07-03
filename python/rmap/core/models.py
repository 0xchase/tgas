# rmap/core/models.py
from typing import List, Optional
from pydantic import BaseModel, Field

class AddressSet(BaseModel):
    """
    A fundamental data structure representing a named set of IPv6 addresses.
    This is a common input/output format for many plugin types.
    """
    name: str
    description: Optional[str] = None
    addresses: List[str] = Field(default_factory=list)
