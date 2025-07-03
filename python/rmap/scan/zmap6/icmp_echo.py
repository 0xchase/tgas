# scan/zmap6/icmp_echo.py

from typing import List, Optional, Any, Dict

from rmap.core.registry import rmap
from .base import BaseZmap6Scanner # Relative import

ICMPV6_ECHO_REQUEST_TYPE = 128

@rmap(kind="scan", name="zmap6_icmp_echo")
class Zmap6ICMPv6EchoScanner(BaseZmap6Scanner):
    """
    zmap6 plugin for performing an ICMPv6 Echo Request (ping) scan.
    """

    name = "Zmap6ICMPv6EchoScanner" # Concrete name
    version = "0.1.0"
    description = "Performs an ICMPv6 Echo Request (ping) scan using zmap6."

    def __init__(
        self,
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
            zmap6_path=zmap6_path,
            rate=rate,
            bandwidth=bandwidth,
            sender_threads=sender_threads,
            probes=probes,
            cooldown_time=cooldown_time,
            extra_args=extra_args,
            progress_bars_enabled=progress_bars_enabled, # Pass to base
            **kwargs
        )

    def _build_specific_zmap6_args(self) -> List[str]:
        return ["--probe-module=icmp6_echoscan"]

    def _get_scan_protocol(self) -> str:
        return "icmpv6"

    def _get_scanned_port_or_type(self) -> int:
        return ICMPV6_ECHO_REQUEST_TYPE

    def _map_zmap_status(self, row: Dict[str, str]) -> str:
        success = row.get("success", "0").strip()
        classification = row.get("classification", "").strip().lower()

        if success == "1":
            if classification == "echo_reply":
                return "responsive"
            else:
                return f"responsive_other_{classification}" if classification else "responsive_other"
        else:
            return "unresponsive"
