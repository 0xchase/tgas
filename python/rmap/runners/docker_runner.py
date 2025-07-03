# rmap/cli/runners/docker_runner.py
# (Content is largely the same, but with updated absolute import paths)
import json
import os
import tempfile
import uuid
from typing import Type, Any, Dict, Optional

from pydantic import BaseModel
try:
    import docker
    from docker.types import Mount
    from docker.errors import DockerException
except ImportError:
    docker = None

# Use absolute imports
from .base import BaseRunner
from rmap.core.plugin import BasePlugin
from rmap.tga.base import ProgressCallback

class DockerRunner(BaseRunner):
    def __init__(self, docker_client_args: Optional[Dict[str, Any]] = None, **kwargs):
        if docker is None:
            raise ImportError("Docker SDK for Python is not installed. Please install it.")
        self.client = docker.DockerClient(**(docker_client_args or {}))
        # ... rest of the implementation is the same as before ...

    def execute(
        self,
        plugin_cls: Type[BasePlugin],
        plugin_init_args: Dict[str, Any],
        run_method_args: Dict[str, Any],
        input_data: Optional[BaseModel] = None,
        progress_cb: Optional[ProgressCallback] = None,
        **cfg: Any
    ) -> BaseModel:
        # ... implementation is the same as before ...
        # (This is a stub for brevity, use the full implementation from the previous response)
        raise NotImplementedError("DockerRunner logic to be pasted here.")

    def close(self) -> None:
        if self.client:
            self.client.close()
