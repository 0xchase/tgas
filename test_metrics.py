#!/usr/bin/env python3
"""
Test script to demonstrate IPv6 toolkit metrics functionality.
This script makes requests to the gRPC server and then fetches metrics.
"""

import asyncio
import aiohttp
import time
import sys
import os

# Add the python directory to the path so we can import ipv6kit
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'python'))

from ipv6kit.core.registry import ipv6kit
from ipv6kit.runners.local_runner import LocalRunner

async def test_metrics():
    """Test the metrics functionality by making various requests."""
    
    print("🚀 Starting IPv6 Toolkit Metrics Test")
    print("=" * 50)
    
    # Test 1: Generate addresses
    print("\n📊 Test 1: Generating IPv6 addresses...")
    try:
        runner = LocalRunner()
        
        # Generate some addresses
        result = await runner.run_plugin(
            "tga", 
            "entropy_ip", 
            {"count": 10, "unique": True}
        )
        
        if result.success:
            print(f"✅ Generated {len(result.data)} addresses")
        else:
            print(f"❌ Generation failed: {result.error}")
            
    except Exception as e:
        print(f"❌ Generation test failed: {e}")
    
    # Test 2: Fetch metrics from Prometheus endpoint
    print("\n📈 Test 2: Fetching metrics from Prometheus endpoint...")
    try:
        async with aiohttp.ClientSession() as session:
            async with session.get('http://localhost:9090/metrics') as response:
                if response.status == 200:
                    metrics_text = await response.text()
                    print("✅ Successfully fetched metrics:")
                    
                    # Parse and display some key metrics
                    lines = metrics_text.split('\n')
                    key_metrics = [
                        'ipv6kit_requests_total',
                        'ipv6kit_addresses_generated_total',
                        'ipv6kit_generation_duration_ms',
                        'ipv6kit_active_generations',
                        'ipv6kit_errors_total'
                    ]
                    
                    for line in lines:
                        for metric in key_metrics:
                            if line.startswith(metric):
                                print(f"   {line}")
                                break
                else:
                    print(f"❌ Failed to fetch metrics: HTTP {response.status}")
                    
    except Exception as e:
        print(f"❌ Metrics fetch failed: {e}")
        print("   Make sure the gRPC server is running with: cargo run --bin ipv6kit -- grpc --addr 0.0.0.0:50051")
    
    # Test 3: Test scan functionality (if available)
    print("\n🔍 Test 3: Testing scan functionality...")
    try:
        # This would require the gRPC server to be running
        print("   Note: Scan testing requires gRPC server to be running")
        print("   Run: cargo run --bin ipv6kit -- grpc --addr 0.0.0.0:50051")
        
    except Exception as e:
        print(f"❌ Scan test failed: {e}")
    
    print("\n" + "=" * 50)
    print("🎯 Metrics Test Complete!")
    print("\nTo see all metrics, visit: http://localhost:9090/metrics")
    print("To start the gRPC server: cargo run --bin ipv6kit -- grpc --addr 0.0.0.0:50051")

if __name__ == "__main__":
    asyncio.run(test_metrics()) 