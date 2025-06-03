# core/plugin.py
import logging
from abc import ABC, abstractmethod
from typing import Generic, TypeVar, Type, Optional, Any, Dict, Union

from pydantic import BaseModel

try:
    import tqdm
    TQDM_AVAILABLE = True
except ImportError:
    TQDM_AVAILABLE = False

_InputT = TypeVar('_InputT', bound=Optional[BaseModel])
_OutputT = TypeVar('_OutputT', bound=BaseModel)

# Define a type for progress bar handles for clarity
ProgressBarHandle = str

class BasePlugin(ABC, Generic[_InputT, _OutputT]):
    """
    The universal abstract base class for all plugins in the ipv6kit ecosystem.
    It defines the core interface and contract for any executable unit.
    Includes basic logging and progress bar management capabilities.
    """
    # --- Core Plugin Attributes (defined by concrete plugins) ---
    name: str # Should be overridden by concrete plugin (or set by registry)
    version: str # Should be overridden
    description: Optional[str] = None

    input_type: Optional[Type[BaseModel]]
    output_type: Type[BaseModel]
    
    # --- Internal Attributes ---
    logger: logging.Logger
    _progress_bars_enabled: bool
    _progress_bars: Dict[ProgressBarHandle, Any] # Stores tqdm instances if enabled
    _next_progress_bar_position: int # For auto-layout of multiple tqdm bars

    def __init__(self, progress_bars_enabled: bool = False, **kwargs: Any):
        """
        Initializes the BasePlugin.

        Args:
            progress_bars_enabled (bool): If True, enables visual progress bars (requires tqdm).
                                          Defaults to False.
            **kwargs: Absorbs any other keyword arguments passed by subclasses during super() call,
                      or from plugin instantiation if not handled by concrete plugin's __init__.
        """
        # Configure logger for this specific plugin instance
        # The actual handlers (e.g., stdout, file) and global log level
        # should be configured by the application's entry point (e.g., CLI).
        self.logger = logging.getLogger(f"{self.__class__.__module__}.{self.__class__.__name__}")

        self._progress_bars_enabled = progress_bars_enabled and TQDM_AVAILABLE
        self._progress_bars = {}
        self._next_progress_bar_position = 0

        if progress_bars_enabled and not TQDM_AVAILABLE:
            self.warning("Progress bars were enabled, but 'tqdm' library is not installed. Progress bars will be disabled.")

    # --- Logging Methods ---
    def info(self, message: str, *args: Any, **kwargs: Any) -> None:
        """Logs an informational message."""
        self.logger.info(message, *args, **kwargs)

    def warning(self, message: str, *args: Any, **kwargs: Any) -> None:
        """Logs a warning message."""
        self.logger.warning(message, *args, **kwargs)

    def error(self, message: str, *args: Any, exc_info: bool = False, **kwargs: Any) -> None:
        """Logs an error message. Set exc_info=True to include exception info."""
        self.logger.error(message, *args, exc_info=exc_info, **kwargs)

    def critical(self, message: str, *args: Any, exc_info: bool = False, **kwargs: Any) -> None:
        """Logs a critical message. Set exc_info=True to include exception info."""
        self.logger.critical(message, *args, exc_info=exc_info, **kwargs)

    def debug(self, message: str, *args: Any, **kwargs: Any) -> None:
        """Logs a debug message."""
        self.logger.debug(message, *args, **kwargs)

    # --- Progress Bar Management Methods ---
    def add_progress_bar(
        self,
        name: str,
        total: Optional[float] = None,
        description: Optional[str] = None,
        unit: str = 'it',
        position: Optional[int] = None,
        leave: bool = True,
        **kwargs: Any
    ) -> Optional[ProgressBarHandle]:
        """
        Adds a new progress bar. Requires 'tqdm' to be installed.

        Args:
            name (str): A unique name/handle for this progress bar.
            total (Optional[float]): The total number of iterations.
            description (Optional[str]): Text displayed next to the progress bar.
            unit (str): The unit for iterations (e.g., 'it', 'B', 'event').
            position (Optional[int]): Specific position for the bar (useful for multiple bars).
                                      If None, auto-assigns position.
            leave (bool): Whether to leave the progress bar displayed when closed.
            **kwargs: Additional keyword arguments to pass to tqdm.tqdm().

        Returns:
            Optional[ProgressBarHandle]: The handle (name) of the created progress bar,
                                         or None if progress bars are disabled or tqdm is unavailable.
        """
        if not self._progress_bars_enabled:
            return None
        if name in self._progress_bars:
            self.warning(f"Progress bar with name '{name}' already exists. Returning existing handle.")
            return name
        
        if position is None:
            position = self._next_progress_bar_position
            self._next_progress_bar_position += 1

        try:
            bar = tqdm.tqdm(
                total=total,
                desc=description or name,
                unit=unit,
                position=position,
                leave=leave,
                dynamic_ncols=True, # Adjust to terminal width
                **kwargs
            )
            self._progress_bars[name] = bar
            return name
        except Exception as e:
            self.error(f"Failed to create progress bar '{name}': {e}", exc_info=True)
            return None

    def update_progress_bar(
        self,
        handle: Optional[ProgressBarHandle],
        advance: float = 1,
        set_description: Optional[str] = None,
        set_postfix: Optional[Dict[str, Any]] = None,
        **kwargs: Any
    ) -> None:
        """
        Updates an existing progress bar.

        Args:
            handle (Optional[ProgressBarHandle]): The handle of the progress bar to update.
            advance (float): Amount to advance the progress bar. Defaults to 1.
            set_description (Optional[str]): New description for the progress bar.
            set_postfix (Optional[Dict[str, Any]]): Dictionary of postfix key-value pairs.
            **kwargs: Additional arguments for tqdm's set_postfix if set_postfix is None.
        """
        if not self._progress_bars_enabled or handle is None or handle not in self._progress_bars:
            return
        
        bar = self._progress_bars[handle]
        if advance > 0:
            bar.update(advance)
        if set_description is not None:
            bar.set_description_str(set_description, refresh=False) # Refresh handled by update or next loop
        if set_postfix is not None:
            bar.set_postfix(set_postfix, refresh=False)
        elif kwargs: # Allow direct postfix from kwargs if set_postfix not used
            bar.set_postfix(kwargs, refresh=False)
        if bar.total is None or bar.n < bar.total : # Avoid refreshing closed or completed bar unnecessarily if not advancing
             bar.refresh()


    def close_progress_bar(self, handle: Optional[ProgressBarHandle]) -> None:
        """
        Closes a specific progress bar.

        Args:
            handle (Optional[ProgressBarHandle]): The handle of the progress bar to close.
        """
        if not self._progress_bars_enabled or handle is None or handle not in self._progress_bars:
            return
        
        bar = self._progress_bars.pop(handle)
        bar.close()
        # Adjust next position if managing positions dynamically, though tqdm handles overlaps.
        # For simplicity, we don't try to reclaim positions here.
        # If a bar at the highest position is closed, _next_progress_bar_position
        # will ensure new bars don't immediately overwrite.

    def close_all_progress_bars(self) -> None:
        """Closes all active progress bars managed by this plugin instance."""
        if not self._progress_bars_enabled:
            return
        
        for name in list(self._progress_bars.keys()): # Iterate over a copy of keys
            self.close_progress_bar(name)
        self._next_progress_bar_position = 0 # Reset position counter

    @abstractmethod
    def run(self, data: Optional[_InputT], **kwargs: Any) -> _OutputT:
        """
        Execute the plugin's main logic.
        'data' is the input, validated against input_type if not None.
        'kwargs' can be used for additional runtime parameters not part of the input model,
        like 'progress_cb' (the abstract one).
        """
        # Ensure that cleanup happens even if run() raises an exception
        # However, putting try/finally here might be too prescriptive.
        # Plugins should manage their resources, including calling close_all_progress_bars.
        ...

    def __del__(self):
        # Attempt to clean up progress bars if the object is garbage collected,
        # though explicit closing is preferred.
        if hasattr(self, '_progress_bars_enabled') and self._progress_bars_enabled:
            if hasattr(self, '_progress_bars') and self._progress_bars:
                self.warning(f"Plugin {self.name} is being deleted with active progress bars. "
                             "Ensure close_all_progress_bars() is called explicitly.")
                # self.close_all_progress_bars() # This can sometimes cause issues during GC
