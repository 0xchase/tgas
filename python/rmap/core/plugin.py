import logging
import pathlib
from typing import Type, Optional, Any, TypeVar
from pydantic import BaseModel

# Generic type for Pydantic models, useful for helper methods
_BaseModelT = TypeVar('_BaseModelT', bound=BaseModel)

class BasePlugin:
    """
    A simplified base class for all rmap plugins.
    Concrete plugins will define public methods that python-fire
    can expose as subcommands.
    """
    # These attributes will be set by the @rmap decorator or by the plugin class itself.
    name: str
    version: str
    description: Optional[str] = None

    def __init__(self, **kwargs: Any):
        """
        Base constructor. Concrete plugins will define their specific
        initialization parameters, which Fire will use to populate CLI flags
        for plugin instantiation.
        
        kwargs here allows subclasses to call super().__init__(**kwargs)
        if they also accept arbitrary kwargs, though it's not strictly necessary
        for this simple base.
        """
        # Initialize a logger for the concrete plugin instance
        self._logger = logging.getLogger(f"{self.__class__.__module__}.{self.__class__.__name__}")
        # Any common initialization for all plugins can go here.
        pass
