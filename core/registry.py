# ipv6kit/core/registry.py
from collections import defaultdict
from typing import Dict, Type, Callable, Any, Optional

# Forward declaration for BasePlugin type hint
from typing import TYPE_CHECKING
if TYPE_CHECKING:
    from .plugin import BasePlugin

PLUGINS: Dict[str, Dict[str, Type["BasePlugin"]]] = defaultdict(dict)
PLUGIN_KINDS = {"dataset", "tga", "scanner", "analyze"}

def ipv6kit(kind: str, name: Optional[str] = None) -> Callable[[Type["BasePlugin"]], Type["BasePlugin"]]:
    """
    Decorator to register a plugin class with the ipv6kit framework.
    """
    if kind not in PLUGIN_KINDS:
        raise ValueError(f"Unknown plugin kind: {kind}. Must be one of {PLUGIN_KINDS}")

    def decorator(cls: Type["BasePlugin"]) -> Type["BasePlugin"]:
        from .plugin import BasePlugin
        if not issubclass(cls, BasePlugin):
            raise TypeError(f"Plugin class {cls.__name__} must extend core.plugin.BasePlugin")
        
        plugin_name = name or cls.__name__
        if plugin_name in PLUGINS[kind]:
            raise ValueError(f"Plugin {kind}/{plugin_name} is already registered. ({PLUGINS[kind][plugin_name]})")

        PLUGINS[kind][plugin_name] = cls
        
        if not hasattr(cls, 'name') or cls.name is None:
            cls.name = plugin_name
        if not hasattr(cls, 'version') or cls.version is None:
            cls.version = "0.1.0"

        return cls
    return decorator

# Helper functions to access the registry remain the same.
