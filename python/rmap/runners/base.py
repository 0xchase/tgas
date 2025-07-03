# rmap/cli/runners/base.py
from abc import ABC, abstractmethod
from typing import Type, Any, Optional, Dict
from pydantic import BaseModel

# Use absolute imports for cross-package dependencies
from rmap.core.plugin import BasePlugin
from rmap.tga.base import ProgressCallback # Progress callback is defined with TGAs

class BaseRunner(ABC):
    @abstractmethod
    def execute(
        self,
        plugin_cls: Type[BasePlugin],
        plugin_init_args: Dict[str, Any],
        run_method_args: Dict[str, Any],
        input_data: Optional[BaseModel] = None,
        progress_cb: Optional[ProgressCallback] = None,
        **cfg: Any
    ) -> BaseModel:
        ...

    def close(self) -> None:
        pass

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
