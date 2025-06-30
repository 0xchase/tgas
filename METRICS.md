# IPv6 Toolkit Metrics

The IPv6 toolkit provides comprehensive metrics for monitoring and observability. All metrics are exposed via Prometheus format at `http://0.0.0.0:9090/metrics` when running the gRPC server.

## Server Metrics

### Basic Server Metrics
- `ipv6kit_server_starts_total` (counter) - Total number of server starts
- `ipv6kit_server_up` (gauge) - Server status (1 = up, 0 = down)

### Request Metrics
- `ipv6kit_requests_total` (counter) - Total requests by status and operation
  - Labels: `status` (success/failed), `operation` (generate/scan/discover)
- `ipv6kit_request_success_rate` (gauge) - Overall request success rate (0.0-1.0)

### Error Metrics
- `ipv6kit_errors_total` (counter) - Total errors by type and operation
  - Labels: `error_type`, `operation`

## Generation Metrics

### Generation Requests
- `ipv6kit_generation_requests_total` (counter) - Total generation requests
  - Labels: `unique` (true/false)

### Generation Performance
- `ipv6kit_addresses_generated_total` (counter) - Total addresses generated
  - Labels: `unique` (true/false)
- `ipv6kit_generation_duration_ms` (histogram) - Generation duration
  - Labels: `unique` (true/false)
- `ipv6kit_generation_rate_aps` (histogram) - Generation rate (addresses per second)
  - Labels: `unique` (true/false)

### Active Operations
- `ipv6kit_active_generations` (gauge) - Number of active generation operations

## Scan Metrics

### Scan Requests
- `ipv6kit_scan_requests_total` (counter) - Total scan requests
  - Labels: `scan_type`, `probe_module`, `dryrun`

### Scan Performance
- `ipv6kit_addresses_scanned_total` (counter) - Total addresses scanned
  - Labels: `scan_type` (icmpv4/icmpv6/link_local)
- `ipv6kit_scan_duration_ms` (histogram) - Scan duration
  - Labels: `scan_type`
- `ipv6kit_scan_rate_aps` (histogram) - Scan rate (addresses per second)
  - Labels: `scan_type`

### ICMPv4 Specific Metrics
- `ipv6kit_icmp4_scans_total` (counter) - Total ICMPv4 scans
- `ipv6kit_icmp4_hosts_total` (counter) - Total hosts scanned via ICMPv4
- `ipv6kit_icmp4_responses_total` (counter) - Total ICMPv4 responses received
- `ipv6kit_icmp4_response_rate` (gauge) - ICMPv4 response rate (0.0-1.0)
- `ipv6kit_active_icmp4_scans` (gauge) - Active ICMPv4 scans

### ICMPv6 Specific Metrics
- `ipv6kit_icmp6_scans_total` (counter) - Total ICMPv6 scans
- `ipv6kit_icmp6_hosts_total` (counter) - Total hosts scanned via ICMPv6
- `ipv6kit_icmp6_responses_total` (counter) - Total ICMPv6 responses received
- `ipv6kit_icmp6_response_rate` (gauge) - ICMPv6 response rate (0.0-1.0)
- `ipv6kit_active_icmp6_scans` (gauge) - Active ICMPv6 scans

### Link-Local Discovery Metrics
- `ipv6kit_link_local_discoveries_total` (counter) - Total link-local discoveries
- `ipv6kit_link_local_hosts_discovered_total` (counter) - Total hosts discovered via link-local
- `ipv6kit_link_local_interface_errors_total` (counter) - Interface errors during link-local discovery
- `ipv6kit_active_link_local_discoveries` (gauge) - Active link-local discoveries

### Active Operations
- `ipv6kit_active_scans` (gauge) - Number of active scan operations

## Discovery Metrics

### Discovery Requests
- `ipv6kit_discovery_requests_total` (counter) - Total discovery requests

### Discovery Performance
- `ipv6kit_addresses_discovered_total` (counter) - Total addresses discovered
- `ipv6kit_discovery_duration_ms` (histogram) - Discovery duration
- `ipv6kit_discovery_rate_aps` (histogram) - Discovery rate (addresses per second)

### Active Operations
- `ipv6kit_active_discoveries` (gauge) - Number of active discovery operations

## System Metrics

### Memory Tracking
- `ipv6kit_memory_tracking_enabled` (gauge) - Memory tracking status (placeholder for future implementation)

## Usage Examples

### Prometheus Query Examples

```promql
# Request success rate over time
rate(ipv6kit_requests_total{status="success"}[5m]) / rate(ipv6kit_requests_total[5m])

# Average scan duration by type
histogram_quantile(0.95, rate(ipv6kit_scan_duration_ms_bucket[5m]))

# Active operations
ipv6kit_active_generations + ipv6kit_active_scans + ipv6kit_active_discoveries

# Error rate by operation
rate(ipv6kit_errors_total[5m])

# Generation throughput
rate(ipv6kit_addresses_generated_total[5m])

# Scan response rates
ipv6kit_icmp4_response_rate
ipv6kit_icmp6_response_rate
```

### Grafana Dashboard

You can create a Grafana dashboard using these metrics to monitor:
- Request throughput and success rates
- Operation durations and performance
- Active operation counts
- Error rates and types
- Scan and generation rates
- Response rates for different scan types

## Configuration

The metrics are automatically enabled when running the gRPC server. The Prometheus endpoint is available at port 9090 by default.

To customize the metrics endpoint, modify the `run_server` function in `cli/src/frontends/grpc.rs`. 