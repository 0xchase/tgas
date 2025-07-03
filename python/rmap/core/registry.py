# rmap/core/registry.py
import logging
from collections import defaultdict
from typing import Dict, Type, Callable, Any, Optional, List

from typing import TYPE_CHECKING
if TYPE_CHECKING:
    from .plugin import BasePlugin

logger = logging.getLogger(__name__)

PLUGINS: Dict[str, Dict[str, Type["BasePlugin"]]] = defaultdict(dict)

def rmap(kind: str, name: Optional[str] = None) -> Callable[[Type["BasePlugin"]], Type["BasePlugin"]]:
    #if kind not in PLUGIN_KINDS:
    #    raise ValueError(f"Unknown plugin kind: '{kind}'. Must be one of {PLUGIN_KINDS}")

    def decorator(cls: Type["BasePlugin"]) -> Type["BasePlugin"]:
        from .plugin import BasePlugin # Local import for safety
        if not issubclass(cls, BasePlugin):
            raise TypeError(
                f"Plugin class {cls.__module__}.{cls.__name__} must extend "
                f"rmap.core.plugin.BasePlugin"
            )
        
        plugin_name_to_register = name or cls.__name__

        if plugin_name_to_register in PLUGINS[kind]:
            logger.warning(
                f"Plugin {kind}/{plugin_name_to_register} is being overridden. "
                f"Original: {PLUGINS[kind][plugin_name_to_register].__module__}, "
                f"New: {cls.__module__}.{cls.__name__}"
            )

        PLUGINS[kind][plugin_name_to_register] = cls

        setattr(cls, 'name', plugin_name_to_register)
        
        if not hasattr(cls, 'version') or getattr(cls, 'version', None) is None:
            setattr(cls, 'version', "0.1.0")

        logger.debug(f"Registered plugin via decorator: {kind}/{plugin_name_to_register} from {cls.__module__}")
        return cls
    return decorator

def get_all_plugins() -> List[Type["BasePlugin"]]:
    all_plugins = []
    for (kind, plugins) in PLUGINS.items():
        for (name, plugin) in plugins.items():
            all_plugins.append(plugin)
    return all_plugins

# Other accessors like get_plugin, get_all_plugins_by_kind can remain if needed internally.
