# rmap/scan/__init__.py
from .base import StaticTGAPlugin, DynamicTGAPlugin

from .det import DetTga
from .entropyip import EntropyIpTga
from .sixforest import SixForestTga
from .sixgan import SixGanTga
from .sixgcvae import SixGcVaeTga
from .sixgen import SixGenTga
from .sixgraph import SixGraphnTga
from .sixscan import SixScanTga
from .sixtree import SixTreeTga
from .sixveclm import SixVecLmTga


__all__ = [
    "DetTga",
    "EntropyIpTga",
    "SixForestTga",
    "SixGanTga",
    "SixGcVaeTga",
    "SixGenTga",
    "SixGraphnTga",
    "SixScanTga",
    "SixTreeTga",
    "SixVecLmTga",
]
