# rmap/analyze/base.py
from abc import abstractmethod
from typing import Any, Dict
from pydantic import BaseModel

from rmap.core.plugin import BasePlugin
from rmap.scan.base import ScanResultSet

class AnalysisReport(BaseModel):
    """Data model for the output of an analysis plugin."""
    title: str
    summary: str
    details: Dict[str, Any]
    source_scan_results_count: int

class AnalyzePlugin(BasePlugin):
    """Analyze some results"""

    @abstractmethod
    def analyze(self, data: ScanResultSet, **kwargs: Any) -> AnalysisReport:
        pass


