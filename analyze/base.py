# ipv6kit/analyze/base.py
from abc import abstractmethod
from typing import Any, Dict
from pydantic import BaseModel

from ipv6kit.core.plugin import BasePlugin
from ipv6kit.scan.base import ScanResultSet

class AnalysisReport(BaseModel):
    """Data model for the output of an analysis plugin."""
    title: str
    summary: str
    details: Dict[str, Any]
    source_scan_results_count: int

class AnalyzerPlugin(BasePlugin[ScanResultSet, AnalysisReport]):
    """Base class for plugins that analyze scan results."""
    input_type = ScanResultSet
    output_type = AnalysisReport

    @abstractmethod
    def run(self, data: ScanResultSet, **kwargs: Any) -> AnalysisReport:
        pass
