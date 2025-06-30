use tonic::{transport::Server, Request, Response, Status};
use std::net::Ipv6Addr;
use tga::{EntropyIpTga, TGA};
use std::collections::HashSet;
use polars::prelude::*;
use serde_json;
use ipnet::IpNet;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use metrics::{counter, histogram, gauge};
use std::time::Instant;
use metrics_exporter_prometheus;

pub mod ipv6kit {
    tonic::include_proto!("ipv6kit");
}

use ipv6kit::ipv6_kit_service_server::{Ipv6KitService, Ipv6KitServiceServer};
use ipv6kit::{
    GenerateRequest, ScanRequest, DiscoverRequest, DataframeResponse,
    ScanType, ProbeModule
};

/// gRPC server implementation for IPv6 toolkit with metrics
#[derive(Default)]
pub struct Ipv6KitServiceImpl {
    metrics: Arc<Mutex<ServerMetrics>>,
}

#[derive(Default)]
struct ServerMetrics {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    active_scans: u64,
    total_addresses_generated: u64,
    total_addresses_scanned: u64,
}

impl Ipv6KitServiceImpl {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(ServerMetrics::default())),
        }
    }

    async fn record_request(&self, success: bool) {
        let mut metrics = self.metrics.lock().await;
        metrics.total_requests += 1;
        if success {
            metrics.successful_requests += 1;
            counter!("ipv6kit_requests_total", 1, "status" => "success");
        } else {
            metrics.failed_requests += 1;
            counter!("ipv6kit_requests_total", 1, "status" => "failed");
        }
    }

    async fn record_scan_metrics(&self, addresses_scanned: u64, duration_ms: u64) {
        let mut metrics = self.metrics.lock().await;
        metrics.total_addresses_scanned += addresses_scanned;
        
        counter!("ipv6kit_addresses_scanned_total", addresses_scanned);
        histogram!("ipv6kit_scan_duration_ms", duration_ms as f64);
    }

    async fn record_generation_metrics(&self, addresses_generated: u64, duration_ms: u64) {
        let mut metrics = self.metrics.lock().await;
        metrics.total_addresses_generated += addresses_generated;
        
        counter!("ipv6kit_addresses_generated_total", addresses_generated);
        histogram!("ipv6kit_generation_duration_ms", duration_ms as f64);
    }
}

#[tonic::async_trait]
impl Ipv6KitService for Ipv6KitServiceImpl {
    async fn generate(
        &self,
        request: Request<GenerateRequest>,
    ) -> Result<Response<DataframeResponse>, Status> {
        let start_time = Instant::now();
        let req = request.into_inner();
        
        // Record active generation
        gauge!("ipv6kit_active_generations", 1.0);
        
        // Use the same seed IPs as in the original implementation
        let seed_ips: Vec<[u8; 16]> = vec![
            Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0001, 0, 0, 0, 0x0001).octets(),
            Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0001, 0, 0, 0, 0x0002).octets(),
            Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0002, 0, 0, 0, 0x0001).octets(),
            Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0002, 0, 0, 0, 0x0002).octets(),
            Ipv6Addr::new(0x2001, 0x0db8, 0x0002, 0x000a, 0, 0, 0, 0x000a).octets(),
            Ipv6Addr::new(0x2001, 0x0db8, 0x0002, 0x000a, 0, 0, 0, 0x000b).octets(),
            Ipv6Addr::new(0x2001, 0x0db8, 0x0002, 0x000b, 0, 0, 0, 0x000a).octets(),
            Ipv6Addr::new(0x2001, 0x0db8, 0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666).octets(),
            Ipv6Addr::new(0x2001, 0x0db8, 0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6667).octets(),
        ];

        let tga = match EntropyIpTga::train(seed_ips) {
            Ok(tga) => tga,
            Err(e) => {
                self.record_request(false).await;
                gauge!("ipv6kit_active_generations", 0.0);
                return Ok(Response::new(DataframeResponse {
                    dataframe_json: "".to_string(),
                    success: false,
                    error: format!("Failed to train model: {e}"),
                }));
            }
        };

        let mut generated = HashSet::new();
        let mut addresses = Vec::new();
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 1_000_000;

        while addresses.len() < req.count as usize {
            let generated_bytes = tga.generate();
            let generated_ip = Ipv6Addr::from(generated_bytes);

            if !req.unique || generated.insert(generated_ip) {
                addresses.push(generated_ip.to_string());
                attempts = 0;
            } else {
                attempts += 1;
                if attempts >= MAX_ATTEMPTS {
                    self.record_request(false).await;
                    gauge!("ipv6kit_active_generations", 0.0);
                    return Ok(Response::new(DataframeResponse {
                        dataframe_json: "".to_string(),
                        success: false,
                        error: format!("Could only generate {}/{} unique addresses after {} attempts", 
                                      addresses.len(), req.count, MAX_ATTEMPTS),
                    }));
                }
            }
        }

        // Create DataFrame from generated addresses
        let df = match DataFrame::new(vec![Series::new("address".into(), addresses.clone()).into()]) {
            Ok(df) => df,
            Err(e) => {
                let _ = self.record_request(false).await;
                let _ = gauge!("ipv6kit_active_generations", 0.0);
                return Err(Status::internal(format!("Failed to create DataFrame: {}", e)));
            }
        };

        let df_json = match serde_json::to_string(&df) {
            Ok(json) => json,
            Err(e) => {
                let _ = self.record_request(false).await;
                let _ = gauge!("ipv6kit_active_generations", 0.0);
                return Err(Status::internal(format!("Failed to serialize DataFrame: {}", e)));
            }
        };

        let duration = start_time.elapsed();
        self.record_request(true).await;
        self.record_generation_metrics(addresses.len() as u64, duration.as_millis() as u64).await;
        gauge!("ipv6kit_active_generations", 0.0);

        Ok(Response::new(DataframeResponse {
            dataframe_json: df_json,
            success: true,
            error: "".to_string(),
        }))
    }

    async fn scan(
        &self,
        request: Request<ScanRequest>,
    ) -> Result<Response<DataframeResponse>, Status> {
        let start_time = Instant::now();
        let req = request.into_inner();
        
        // Record active scan
        gauge!("ipv6kit_active_scans", 1.0);
        
        // Parse target
        let target = match parse_target(&req.target) {
            Ok(target) => target,
            Err(e) => {
                let _ = self.record_request(false).await;
                let _ = gauge!("ipv6kit_active_scans", 0.0);
                return Ok(Response::new(DataframeResponse {
                    dataframe_json: "".to_string(),
                    success: false,
                    error: format!("Failed to parse target: {}", e),
                }));
            }
        };

        let results = match (req.scan_type, target) {
            (0, Target::Network(IpNet::V4(net))) => { // ScanType::Icmpv4
                scan::icmp6::icmp4_scan(net)
            }
            (1, Target::Network(IpNet::V6(net))) => { // ScanType::Icmpv6
                scan::icmp6::icmp6_scan(net)
            }
            (2, _) => { // ScanType::LinkLocal
                let results: Vec<scan::icmp6::ProbeResult> = match scan::link_local::discover_all_ipv6_link_local() {
                    Ok(hosts) => {
                        hosts.into_iter().map(|host| {
                            scan::icmp6::ProbeResult {
                                addr: std::net::IpAddr::V6(host),
                                rtt: std::time::Duration::from_millis(0), // Placeholder
                            }
                        }).collect()
                    }
                    Err(e) => {
                        let _ = self.record_request(false).await;
                        let _ = gauge!("ipv6kit_active_scans", 0.0);
                        return Ok(Response::new(DataframeResponse {
                            dataframe_json: "".to_string(),
                            success: false,
                            error: format!("Discovery failed: {}", e),
                        }));
                    }
                };
                results
            }
            _ => {
                let _ = self.record_request(false).await;
                let _ = gauge!("ipv6kit_active_scans", 0.0);
                return Ok(Response::new(DataframeResponse {
                    dataframe_json: "".to_string(),
                    success: false,
                    error: "Unsupported scan type and target combination".to_string(),
                }));
            }
        };

        // Create DataFrame from scan results
        let addresses: Vec<String> = results.iter().map(|r| r.addr.to_string()).collect();
        let rtts: Vec<u64> = results.iter().map(|r| r.rtt.as_millis() as u64).collect();

        let df = match DataFrame::new(vec![
            Series::new("address".into(), addresses.clone()).into(),
            Series::new("rtt_ms".into(), rtts).into(),
        ]) {
            Ok(df) => df,
            Err(e) => {
                let _ = self.record_request(false).await;
                let _ = gauge!("ipv6kit_active_scans", 0.0);
                return Err(Status::internal(format!("Failed to create DataFrame: {}", e)));
            }
        };

        let df_json = match serde_json::to_string(&df) {
            Ok(json) => json,
            Err(e) => {
                let _ = self.record_request(false).await;
                let _ = gauge!("ipv6kit_active_scans", 0.0);
                return Err(Status::internal(format!("Failed to serialize DataFrame: {}", e)));
            }
        };

        let duration = start_time.elapsed();
        self.record_request(true).await;
        self.record_scan_metrics(addresses.len() as u64, duration.as_millis() as u64).await;
        gauge!("ipv6kit_active_scans", 0.0);

        Ok(Response::new(DataframeResponse {
            dataframe_json: df_json,
            success: true,
            error: "".to_string(),
        }))
    }

    async fn discover(
        &self,
        _request: Request<DiscoverRequest>,
    ) -> Result<Response<DataframeResponse>, Status> {
        let start_time = Instant::now();
        
        // Record active discovery
        gauge!("ipv6kit_active_discoveries", 1.0);
        
        // For now, discover is the same as link-local scan
        // This can be extended with more sophisticated discovery logic
        
        let results: Vec<scan::icmp6::ProbeResult> = match scan::link_local::discover_all_ipv6_link_local() {
            Ok(hosts) => {
                hosts.into_iter().map(|host| {
                    scan::icmp6::ProbeResult {
                        addr: std::net::IpAddr::V6(host),
                        rtt: std::time::Duration::from_millis(0), // Placeholder
                    }
                }).collect()
            }
            Err(e) => {
                let _ = self.record_request(false).await;
                let _ = gauge!("ipv6kit_active_discoveries", 0.0);
                return Ok(Response::new(DataframeResponse {
                    dataframe_json: "".to_string(),
                    success: false,
                    error: format!("Discovery failed: {}", e),
                }));
            }
        };

        // Create DataFrame from discovery results
        let addresses: Vec<String> = results.iter().map(|r| r.addr.to_string()).collect();
        let rtts: Vec<u64> = results.iter().map(|r| r.rtt.as_millis() as u64).collect();

        let df = match DataFrame::new(vec![
            Series::new("address".into(), addresses.clone()).into(),
            Series::new("rtt_ms".into(), rtts).into(),
        ]) {
            Ok(df) => df,
            Err(e) => {
                let _ = self.record_request(false).await;
                let _ = gauge!("ipv6kit_active_discoveries", 0.0);
                return Err(Status::internal(format!("Failed to create DataFrame: {}", e)));
            }
        };

        let df_json = match serde_json::to_string(&df) {
            Ok(json) => json,
            Err(e) => {
                let _ = self.record_request(false).await;
                let _ = gauge!("ipv6kit_active_discoveries", 0.0);
                return Err(Status::internal(format!("Failed to serialize DataFrame: {}", e)));
            }
        };

        let duration = start_time.elapsed();
        self.record_request(true).await;
        self.record_scan_metrics(addresses.len() as u64, duration.as_millis() as u64).await;
        gauge!("ipv6kit_active_discoveries", 0.0);

        Ok(Response::new(DataframeResponse {
            dataframe_json: df_json,
            success: true,
            error: "".to_string(),
        }))
    }
}

#[derive(Debug)]
enum Target {
    SingleIp(IpAddr),
    Network(IpNet),
    Hostname(String, Vec<IpAddr>),
}

fn parse_target(input: &str) -> Result<Target, String> {
    // Try parsing as IP address first
    if let Ok(ip) = input.parse::<IpAddr>() {
        return Ok(Target::SingleIp(ip));
    }

    // Try parsing as CIDR network
    if let Ok(net) = input.parse::<IpNet>() {
        return Ok(Target::Network(net));
    }

    // For now, return error for hostnames (DNS resolution not implemented)
    Err(format!("Could not parse target: {}", input))
}

/// gRPC client for the IPv6 toolkit
pub struct GrpcClient {
    client: ipv6kit::ipv6_kit_service_client::Ipv6KitServiceClient<tonic::transport::Channel>,
}

impl GrpcClient {
    pub async fn new(addr: String) -> Result<Self, tonic::transport::Error> {
        let client = ipv6kit::ipv6_kit_service_client::Ipv6KitServiceClient::connect(addr).await?;
        Ok(GrpcClient { client })
    }

    pub async fn generate(
        &mut self,
        count: u32,
        unique: bool,
    ) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let request = GenerateRequest { count, unique };
        let response = self.client.generate(request).await?;
        let response = response.into_inner();
        
        if !response.success {
            return Err(response.error.into());
        }

        let df: DataFrame = serde_json::from_str(&response.dataframe_json)?;
        Ok(df)
    }

    pub async fn scan(
        &mut self,
        target: String,
        scan_type: i32,
        rate: u32,
        probes: u32,
        max_runtime: u32,
        cooldown_time: u32,
        seed: u64,
        source_port: String,
        source_ip: String,
        interface: String,
        probe_module: i32,
        dryrun: bool,
    ) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let request = ScanRequest {
            target,
            scan_type,
            rate,
            probes,
            max_runtime,
            cooldown_time,
            seed,
            source_port,
            source_ip,
            interface,
            probe_module,
            dryrun,
        };
        
        let response = self.client.scan(request).await?;
        let response = response.into_inner();
        
        if !response.success {
            return Err(response.error.into());
        }

        let df: DataFrame = serde_json::from_str(&response.dataframe_json)?;
        Ok(df)
    }

    pub async fn discover(&mut self) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let request = DiscoverRequest {};
        let response = self.client.discover(request).await?;
        let response = response.into_inner();
        
        if !response.success {
            return Err(response.error.into());
        }

        let df: DataFrame = serde_json::from_str(&response.dataframe_json)?;
        Ok(df)
    }
}

/// Run the gRPC server with Prometheus metrics
pub async fn run_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse()?;
    let service = Ipv6KitServiceImpl::new();
    
    // Initialize metrics
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], 9090))
        .install()
        .expect("Failed to install Prometheus metrics exporter");
    
    println!("Starting gRPC server on {}", addr);
    println!("Prometheus metrics available at http://0.0.0.0:9090/metrics");
    
    Server::builder()
        .add_service(Ipv6KitServiceServer::new(service))
        .serve(addr)
        .await?;
    
    Ok(())
}

/// Execute a remote command using the gRPC client
pub async fn execute_remote_command(
    server_addr: &str,
    command: &str,
    args: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GrpcClient::new(server_addr.to_string()).await?;
    
    match command {
        "generate" => {
            let count = args.iter()
                .position(|a| a == "--count")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(10);
            let unique = args.contains(&"--unique".to_string());
            
            let df = client.generate(count, unique).await?;
            println!("Generated DataFrame:");
            println!("{:?}", df);
        }
        "scan" => {
            // Parse scan arguments (simplified for now)
            let target = args.get(0).cloned().unwrap_or_else(|| "127.0.0.1".to_string());
            let scan_type = ScanType::Icmpv4; // Default
            let rate = 10000;
            let probes = 1;
            let max_runtime = 0;
            let cooldown_time = 8;
            let seed = 0;
            let source_port = "".to_string();
            let source_ip = "".to_string();
            let interface = "".to_string();
            let probe_module = ProbeModule::TcpSynScan;
            let dryrun = false;
            
            let df = client.scan(
                target, scan_type.into(), rate, probes, max_runtime, cooldown_time,
                seed, source_port, source_ip, interface, probe_module.into(), dryrun
            ).await?;
            println!("Scan DataFrame:");
            println!("{:?}", df);
        }
        "discover" => {
            let df = client.discover().await?;
            println!("Discovery DataFrame:");
            println!("{:?}", df);
        }
        _ => {
            return Err(format!("Unknown command: {}", command).into());
        }
    }
    
    Ok(())
} 