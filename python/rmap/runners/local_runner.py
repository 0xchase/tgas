# rmap/cli/runners/local_runner.py
from typing import Type, Any, Dict, Optional
from pydantic import BaseModel

from .base import BaseRunner
from rmap.core.plugin import BasePlugin
from rmap.tga.base import ProgressCallback

class LocalRunner(BaseRunner):
    def execute(
        self,
        plugin_cls: Type[BasePlugin],
        plugin_init_args: Dict[str, Any],
        run_method_args: Dict[str, Any],
        input_data: Optional[BaseModel] = None,
        progress_cb: Optional[ProgressCallback] = None,
        **cfg: Any
    ) -> BaseModel:
        """Executes the plugin in the current Python interpreter."""
        plugin_instance = plugin_cls(**plugin_init_args)
        
        combined_run_args = {}
        if input_data is not None:
            combined_run_args['data'] = input_data
        
        combined_run_args.update(run_method_args)

        if progress_cb:
            combined_run_args['progress_cb'] = progress_cb
        
        result = plugin_instance.run(**combined_run_args)
        
        if isinstance(result, dict) and issubclass(plugin_cls.output_type, BaseModel):
             return plugin_cls.output_type.model_validate(result)

        return result
