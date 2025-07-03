# scan/zmap6/tcp_syn.py

from typing import List, Optional, Any, Dict
from pydantic import validator # For port validation

from rmap.core.registry import rmap
from .base import BaseZmap6Scanner # Relative import

@rmap(kind="scan", name="zmap6_tcp_syn")
class Zmap6TCPSYNScanner(BaseZmap6Scanner):
    """
    zmap6 plugin for performing a TCP SYN scan.
    """

    name = "Zmap6TCPSYNScanner" # Concrete name
    version = "0.1.0"
    description = "Performs a TCP SYN scan for a specific port using zmap6."

    port: int

    def __init__(
        self,
        port: int,
        zmap6_path: str = "zmap6",
        rate: Optional[int] = None,
        bandwidth: Optional[str] = None,
        sender_threads: Optional[int] = 1,
        probes: Optional[int] = 1,
        cooldown_time: Optional[int] = None,
        extra_args: Optional[List[str]] = None,
        progress_bars_enabled: bool = False, # Accept from config
        **kwargs: Any
    ):
        super().__init__(
            zmap6_path=zmap6_path, rate=rate, bandwidth=bandwidth,
            sender_threads=sender_threads, probes=probes,
            cooldown_time=cooldown_time, extra_args=extra_args,
            progress_bars_enabled=progress_bars_enabled, # Pass to base
            **kwargs
        )
        self.port = port

    def _port_must_be_valid(cls, v: int) -> int:
        if not (1 <= v <= 65535):
            raise ValueError("Port must be between 1 and 65535")
        return v

    def _build_specific_zmap6_args(self) -> List[str]:
        return ["-p", str(self.port)]

    def _get_scan_protocol(self) -> str:
        return "tcp"

    def _get_scanned_port_or_type(self) -> int:
        return self.port

    def _map_zmap_status(self, row: Dict[str, str]) -> str:
        success = row.get("success", "0").strip()
        classification = row.get("classification", "").strip().lower()

        if success == "1":
            if classification == "synack":
                return "open"
            elif classification == "rst":
                return "closed"
            else:
                return f"responsive_other_{classification}" if classification else "responsive_other"
        else:
            return "filtered"
