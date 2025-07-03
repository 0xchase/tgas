# rmap

A modular network scanner and analyzer focused on IPv6 with first-class support for Target Generation Algorithms (TGAs).

## Overview

Some overview paragraph goes here.

## Features

### Scanning

TODO: Support for various probe types.

- Run as remote scanning server

### Analysis

TODO: entropy, dispersion, subnet, classification, predicate filtering

### Target Generation Algorithms (TGAs)

#### Statistical Methods
- **6Gen (2017)**: Clusters seed addresses by Hamming distance and outputs unobserved neighbors
- **6Graph (2022)**: Constructs co-occurrence graphs of address segments and recombines frequent subgraphs
- **6Forest (2022)**: Builds multiple space-partitioning trees to cover diverse seed patterns
- **DET (2022)**: Splits on highest-entropy bits for maximal variability
- **Entropy/IP (2016)**: Measures nybble-level entropy and builds Bayesian models

#### Machine Learning Methods
- **6GCVAE (2020)**: Gated-CNN variational autoencoder for address generation
- **6VecLM (2021)**: Transformer language model treating address blocks as tokens
- **6GAN (2021)**: Clustered GANs with alias-aware rewards
- **6Tree (2019)**: Hierarchical space tree with dynamic drilling
- **6Scan (2023)**: Region-based scanning with continuous target list updates

## Quick Start

### Installation

```bash
snap install rmap
```

### Basic Usage

```bash
# Generate IPv6 addresses using TGAs
rmap generate --count 1000 --unique

# Scan a network range
rmap scan 2001:db8::/64 --scan-type icmpv6 --rate 1000

# Analyze address patterns
rmap analyze addresses.txt entropy --unique

# Start gRPC server for remote access
rmap serve --addr 0.0.0.0:50051
```

## Command-Line Reference

### `generate`

Generate IPv6 addresses using various TGAs:

```bash
rmap generate [OPTIONS]
  -n, --count <COUNT>    Number of addresses to generate [default: 10]
  -u, --unique          Ensure generated addresses are unique
```

### `scan`
Perform network scanning with extensive configuration options:

```bash
rmap scan [OPTIONS] [TARGET]
  [TARGET]              Target specification (IP, hostname, or CIDR range)
  -s, --scan-type       Type of scan: icmpv4, icmpv6, link_local [default: icmpv4]
  -I, --input-file      Input file containing targets (one per line)
  -b, --blocklist-file  File containing CIDR ranges to exclude
  -w, --allowlist-file  File containing CIDR ranges to include
  -n, --max-targets     Maximum number of targets to probe
  -r, --rate            Send rate in packets per second [default: 10000]
  -P, --probes          Number of probes per target [default: 1]
  -t, --max-runtime     Maximum runtime in seconds
  -c, --cooldown-time   Cooldown time in seconds [default: 8]
  -e, --seed            Random seed for target selection
  -S, --source-ip       Source IP address(es) to use
  -i, --interface       Network interface to use
  -M, --probe-module    Probe type: tcp_syn_scan, icmp_echo_scan, udp_scan
```

### `analyze`
Analyze address datasets with various metrics:

```bash
rmap analyze [OPTIONS] <FILE> <COMMAND>
  <FILE>                Path to file containing data to analyze
  
  Commands:
    dispersion          Address space dispersion metrics
    entropy             Information entropy analysis
    subnets             Subnet distribution analysis
    counts              Count addresses matching each predicate
  
  Options:
    -f, --field         Column name to select from input data
    --include           Include addresses matching these predicates
    --exclude           Exclude addresses matching these predicates
    -u, --unique        Remove duplicate addresses before analysis
```

### `serve`
Start gRPC server for remote command execution:

```bash
rmap serve [OPTIONS]
  -a, --addr            Server address to bind to [default: 127.0.0.1:50051]
  -m, --metrics-port    Prometheus metrics port [default: 9090]
```

## Metrics and Monitoring

The rmap grpc server reports various metrics over opentelemetry for monitoring and observability with grafana.

### Metrics

- `rmap_requests_total` - Total requests by status and operation
- `rmap_addresses_generated_total` - Total addresses generated
- `rmap_addresses_scanned_total` - Total addresses scanned
- `rmap_scan_duration_ms` - Scan duration histograms
- `rmap_generation_rate_aps` - Generation rate (addresses per second)
- `rmap_active_generations` - Active generation operations
- `rmap_active_scans` - Active scan operations
- `rmap_errors_total` - Error counts by type

### Accessing Metrics

When running the gRPC server, metrics are exposed via Prometheus format at `http://0.0.0.0:9090/metrics`.

## Development

Developing a plugin for rmap is easy.

### Building

```bash
cargo build
cargo test
```

### Adding Plugins

TODO: Explain how the plugins work.

## Contributing

TODO: Plugin contribution guide

## Citation

Perhaps I'll publish a paper people can cite.

```bibtex
TODO: Publish a paper lol
```

## Acknowledgments

Breakerspace, University of Maryland, contributors, etc