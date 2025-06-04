# ipv6kit/scan/__init__.py
from .base import StaticTGAPlugin, DynamicTGAPlugin
from .gan import SixGanTga

__all__ = [
    "SixGanTga",
]
