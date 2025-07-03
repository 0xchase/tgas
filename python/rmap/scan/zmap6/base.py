# scan/zmap6/base.py

import subprocess
import tempfile
import os
import csv
import datetime
from abc import ABC, abstractmethod
from typing import List, Optional, Any, Dict

# Assuming 'rmap' is the top-level package visible in PYTHONPATH
from rmap.core.models import AddressSet
from rmap.scan.base import ScanPlugin, ScanResult, ScanResultSet # ScannerPlugin is from scan.base

class BaseZmap6Scanner(ScanPlugin, ABC):
    """
    Abstract base class for ScannerPlugins that use zmap6.
    It handles common tasks like target file creation, zmap6 execution,
    temporary file cleanup, and basic CSV parsing.
    """
    # Class attributes like name, version, description are typically overridden by concrete plugins.
    # If not, these serve as defaults or placeholders.
    name: str = "BaseZmap6Scanner"
    version: str = "0.1.0"
    description: Optional[str] = "Base class for zmap6 scanners."

    zmap6_path: str
    rate: Optional[int]
    bandwidth: Optional[str]
    sender_threads: Optional[int]
    probes: Optional[int]
    cooldown_time: Optional[int]
    #extra_args: List[str]

    _ZMAP6_EXPECTED_OUTPUT_FIELDS = "saddr,classification,success,repeat,cooldown"

    def __init__(
        self,
        zmap6_path: str = "zmap6",
        rate: Optional[int] = None,
        bandwidth: Optional[str] = None,
        sender_threads: Optional[int] = 1,
        probes: Optional[int] = 1,
        cooldown_time: Optional[int] = None,
        #extra_args: Optional[List[str]] = None,
        # Add progress_bars_enabled to pass to super if ScannerPlugin expects it
        progress_bars_enabled: bool = False,
        **kwargs: Any, # Catches other args for super or future use
    ):
        # Pass progress_bars_enabled to ScannerPlugin (which inherits from BasePlugin)
        super().__init__(progress_bars_enabled=progress_bars_enabled, **kwargs)
        self.zmap6_path = zmap6_path
        self.rate = rate
        self.bandwidth = bandwidth
        self.sender_threads = sender_threads
        self.probes = probes
        self.cooldown_time = cooldown_time
        #self.extra_args = extra_args or []

    @abstractmethod
    def _build_specific_zmap6_args(self) -> List[str]:
        """
        Subclasses must implement this to provide zmap6 arguments
        specific to their scan type (e.g., probe module, port).
        """
        ...

    @abstractmethod
    def _get_scan_protocol(self) -> str:
        """
        Subclasses must implement this to return the protocol string
        (e.g., "tcp", "udp", "icmpv6").
        """
        ...

    @abstractmethod
    def _get_scanned_port_or_type(self) -> int:
        """
        Subclasses must implement this to return the target port number
        or an ICMP type/code if applicable.
        """
        ...

    @abstractmethod
    def _map_zmap_status(self, row: Dict[str, str]) -> str:
        """
        Subclasses must implement this to map zmap6 output fields
        (from a parsed CSV row dictionary) to a ScanResult status string.
        """
        ...
    
    def _get_scan_name_suffix(self) -> str:
        # Use self.name (which should be set by the concrete class) for better identification
        concrete_plugin_name = getattr(self, 'name', 'BaseZmap6Scanner').lower().replace("scanner", "")
        return f"{concrete_plugin_name}_{self._get_scan_protocol()}_{self._get_scanned_port_or_type()}"

    def scan(self, data: AddressSet, **kwargs: Any) -> ScanResultSet:
        plugin_display_name = getattr(self, 'name', 'Zmap6Scan') # Use concrete plugin's name

        if not data.addresses:
            self.info(f"No addresses provided to scan for {plugin_display_name}.")
            if progress_cb: # Still support the abstract progress_cb if provided
                progress_cb(f"{plugin_display_name}_info", {"message": "No addresses provided to scan.", "count": 0})
            return ScanResultSet(results=[], scan_name=self._get_scan_name_suffix())

        scan_results: List[ScanResult] = []
        
        main_pb_handle = None
        if self._progress_bars_enabled: # Check if base plugin has progress bars enabled
             main_pb_handle = self.add_progress_bar(
                name=f"{plugin_display_name}_overall",
                description=f"{plugin_display_name} preparing...",
                total=len(data.addresses) + 3 # 3 stages: setup, run, parse
            )

        with tempfile.NamedTemporaryFile(mode="w+", delete=False, suffix="_targets.txt") as target_file, \
             tempfile.NamedTemporaryFile(mode="w+", delete=False, suffix="_results.csv") as output_file:
            
            target_file_path = target_file.name
            output_file_path = output_file.name
            
            try:
                self.update_progress_bar(main_pb_handle, description=f"{plugin_display_name} writing targets...")
                for addr in data.addresses:
                    target_file.write(f"{addr}\n")
                target_file.flush()
                self.update_progress_bar(main_pb_handle, advance=1)


                self.info(f"Target file created at {target_file_path} with {len(data.addresses)} addresses.")
                if progress_cb:
                    progress_cb(f"{plugin_display_name}_setup", {"message": f"Target file created", "address_count": len(data.addresses)})

                cmd = [self.zmap6_path]
                if self.rate is not None: cmd.extend(["-r", str(self.rate)])
                if self.bandwidth is not None: cmd.extend(["-B", self.bandwidth])
                # ... (other common args) ...
                
                cmd.extend(self._build_specific_zmap6_args())
                cmd.extend([
                    "--ipv6-target-file", target_file_path,
                    "-o", output_file_path,
                    "-f", self._ZMAP6_EXPECTED_OUTPUT_FIELDS,
                    "--output-filter=" 
                ])
                
                #cmd.extend(self.extra_args)

                self.info(f"Executing zmap6 command: {' '.join(cmd)}")
                if progress_cb: progress_cb(f"{plugin_display_name}_start", {"command": " ".join(cmd)})
                self.update_progress_bar(main_pb_handle, description=f"{plugin_display_name} running zmap6...")
                
                process = subprocess.run(cmd, capture_output=True, text=True, check=False)
                self.update_progress_bar(main_pb_handle, advance=1)

                if progress_cb: progress_cb(f"{plugin_display_name}_process_complete", {"return_code": process.returncode})

                if process.returncode != 0:
                    error_message = f"{plugin_display_name} (zmap6) execution failed with code {process.returncode}.\nStderr: {process.stderr}\nStdout: {process.stdout}"
                    self.error(error_message) # Use new logging method
                    if progress_cb: progress_cb(f"{plugin_display_name}_error", {"message": error_message})
                else:
                    self.info(f"zmap6 execution completed successfully for {plugin_display_name}.")

                self.update_progress_bar(main_pb_handle, description=f"{plugin_display_name} parsing results...")
                try:
                    with open(output_file_path, 'r', newline='') as csvfile:
                        reader = csv.DictReader(csvfile)
                        # Create a sub-progress bar for parsing if many results expected potentially
                        # For now, just update the main one implicitly by its total.
                        for row_idx, row in enumerate(reader):
                            try:
                                scanned_ip = row.get("saddr", "").strip()
                                if not scanned_ip: continue
                                status = self._map_zmap_status(row)
                                scan_results.append(
                                    ScanResult(
                                        address=scanned_ip,
                                        port=self._get_scanned_port_or_type(),
                                        protocol=self._get_scan_protocol(),
                                        status=status,
                                        timestamp=datetime.datetime.now(datetime.timezone.utc)
                                    )
                                )
                            except Exception as e:
                                err_msg = f"Error parsing row {row_idx} for {plugin_display_name}: {row} - {e}"
                                self.warning(err_msg) # Use new logging method
                                if progress_cb: progress_cb(f"{plugin_display_name}_parsing_row_error", {"error": err_msg})
                except FileNotFoundError:
                    msg = f"zmap6 output file not found for {plugin_display_name}: {output_file_path}"
                    self.error(msg)
                    if progress_cb: progress_cb(f"{plugin_display_name}_error", {"message": msg})
                except Exception as e:
                    msg = f"Error reading/parsing zmap6 output file for {plugin_display_name} {output_file_path}: {e}"
                    self.error(msg)
                    if progress_cb: progress_cb(f"{plugin_display_name}_error", {"message": msg})
                self.update_progress_bar(main_pb_handle, advance=1, description=f"{plugin_display_name} parsing complete.")
            finally:
                try:
                    os.remove(target_file_path)
                    os.remove(output_file_path)
                except OSError as e:
                    warn_msg = f"Could not remove temporary files for {plugin_display_name}: {e}"
                    self.warning(warn_msg)
                    if progress_cb: progress_cb(f"{plugin_display_name}_cleanup_error", {"error": str(e)})
        
        self.info(f"{plugin_display_name} scan complete. Found {len(scan_results)} results.")
        if progress_cb: progress_cb(f"{plugin_display_name}_scan_complete", {"results_count": len(scan_results)})
        
        self.close_progress_bar(main_pb_handle) # Close the main progress bar

        return ScanResultSet(results=scan_results, scan_name=self._get_scan_name_suffix())
