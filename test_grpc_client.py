#!/usr/bin/env python3
"""
Simple gRPC client to test IPv6 toolkit and generate metrics
"""

import sys
import os

# Add the python directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'python'))

try:
    from ipv6kit.core.registry import ipv6kit
    from ipv6kit.runners.local_runner import LocalRunner
    
    print("Testing IPv6 toolkit gRPC client...")
    
    # Test generation
    runner = LocalRunner()
    result = runner.generate(count=10, unique=True)
    print(f"Generated {len(result)} addresses")
    
    # Test scan
    result = runner.scan(target="127.0.0.1/30", scan_type="icmpv4")
    print(f"Scan completed, found {len(result)} responsive hosts")
    
    print("Metrics should now be available in Prometheus!")
    
except ImportError as e:
    print(f"Import error: {e}")
    print("Make sure you're in the correct directory and the Python modules are available")
except Exception as e:
    print(f"Error: {e}") 