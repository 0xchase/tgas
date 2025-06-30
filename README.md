# IPv6 Toolkit

A comprehensive toolkit for IPv6 network scanning, analysis, and address generation.

## Features

- **Address Generation**: Generate IPv6 addresses using various algorithms (TGA - Target Generation Algorithms)
- **Network Scanning**: ICMPv4, ICMPv6, and link-local discovery scanning
- **Analysis**: Comprehensive analysis of IPv6 address sets including entropy, dispersion, and subnet analysis
- **gRPC Server**: Remote execution capabilities with full metrics and monitoring
- **Plugin System**: Extensible plugin architecture for custom algorithms and analysis

## Quick Start

### Installation

```bash
git clone <repository-url>
cd ipv6kit
cargo build --release
```

### Basic Usage

```bash
# Generate IPv6 addresses
cargo run -- generate --count 100 --unique

# Scan a network
cargo run -- scan --target 2001:db8::/64 --scan-type icmpv6

# Analyze address set
cargo run -- analyze --input addresses.txt --analysis entropy

# Start gRPC server
cargo run -- grpc --addr 0.0.0.0:50051
```

## Metrics and Monitoring

The IPv6 toolkit provides comprehensive metrics for monitoring and observability. When running the gRPC server, metrics are exposed via Prometheus format at `http://0.0.0.0:9090/metrics`.

### Key Metrics

- **Request Metrics**: Total requests, success rates, and error counts
- **Performance Metrics**: Generation and scan rates, durations, and throughput
- **Operation Metrics**: Active operations, response rates, and discovery statistics
- **Error Metrics**: Detailed error tracking by type and operation

### Available Metrics

- `ipv6kit_requests_total` - Total requests by status and operation
- `ipv6kit_addresses_generated_total` - Total addresses generated
- `ipv6kit_addresses_scanned_total` - Total addresses scanned
- `ipv6kit_scan_duration_ms` - Scan duration histograms
- `ipv6kit_generation_rate_aps` - Generation rate (addresses per second)
- `ipv6kit_active_generations` - Active generation operations
- `ipv6kit_active_scans` - Active scan operations
- `ipv6kit_errors_total` - Error counts by type

### Testing Metrics

Run the metrics test script to verify functionality:

```bash
python3 test_metrics.py
```

For detailed metrics documentation, see [METRICS.md](METRICS.md).

## Architecture

The toolkit is organized into several modules:

- **cli/**: Command-line interface and gRPC server
- **scan/**: Network scanning capabilities
- **analyze/**: Address analysis and statistics
- **tga/**: Target Generation Algorithms
- **plugin/**: Plugin system infrastructure
- **python/**: Python bindings and extensions

## Development

### Building

```bash
cargo build
cargo test
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test -p scan
cargo test -p analyze
```

### Adding Plugins

Plugins can be added by implementing the appropriate traits:

- `TGA` for address generation algorithms
- `Predicate` for address classification
- `AbsorbField` for analysis algorithms

## License

[Add your license information here] 