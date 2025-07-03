use crate::frontends::cli;
use indicatif::{ProgressBar, ProgressStyle};
use ipnet::IpNet;
use metrics::{counter, decrement_gauge, gauge, histogram, increment_gauge};
use metrics_exporter_prometheus;
use polars::prelude::*;
use serde_json;
use std::net::IpAddr;
use std::net::Ipv6Addr;
use std::sync::Arc;
use std::time::Instant;
use tga::{EntropyIpTga, TGA};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status, transport::Server};
use tracing::{Level, info, span};

pub mod rmap {
    tonic::include_proto!("rmap");
}

use rmap::rmap_service_server::{RmapService, RmapServiceServer};
use rmap::{
    DataframeResponse, DiscoverRequest, ExecuteCommandRequest, GenerateRequest, ScanRequest,
};

#[derive(Default)]
pub struct RmapServiceImpl {
    metrics: Arc<Mutex<ServerMetrics>>,
}

#[derive(Default)]
struct ServerMetrics {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_addresses_generated: u64,
    total_addresses_scanned: u64,
    total_addresses_discovered: u64,
}

impl RmapServiceImpl {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(ServerMetrics::default())),
        }
    }

    async fn record_request(&self, success: bool, operation: &'static str) {
        let mut metrics = self.metrics.lock().await;
        metrics.total_requests += 1;
        if success {
            metrics.successful_requests += 1;
            counter!("rmap_requests_total", 1, "status" => "success", "operation" => operation);
        } else {
            metrics.failed_requests += 1;
            counter!("rmap_requests_total", 1, "status" => "failed", "operation" => operation);
        }
        let success_rate = if metrics.total_requests > 0 {
            metrics.successful_requests as f64 / metrics.total_requests as f64
        } else {
            0.0
        };
        gauge!("rmap_request_success_rate", success_rate);
    }

    async fn record_error(&self, error_type: &'static str, operation: &'static str) {
        counter!("rmap_errors_total", 1, "error_type" => error_type, "operation" => operation);
    }
}

#[tonic::async_trait]
impl RmapService for RmapServiceImpl {
    async fn generate(
        &self,
        _request: Request<GenerateRequest>,
    ) -> Result<Response<DataframeResponse>, Status> {
        let _span = span!(Level::INFO, "grpc_generate").entered();

        let result = Err(Status::unimplemented("Use ExecuteCommand for all commands"));

        info!("Generate command completed");
        result
    }

    async fn scan(
        &self,
        _request: Request<ScanRequest>,
    ) -> Result<Response<DataframeResponse>, Status> {
        let _span = span!(Level::INFO, "grpc_scan").entered();

        let result = Err(Status::unimplemented("Use ExecuteCommand for all commands"));

        info!("Scan command completed");
        result
    }

    async fn discover(
        &self,
        _request: Request<DiscoverRequest>,
    ) -> Result<Response<DataframeResponse>, Status> {
        let _span = span!(Level::INFO, "grpc_discover").entered();

        let result = Err(Status::unimplemented("Use ExecuteCommand for all commands"));

        info!("Discover command completed");
        result
    }

    async fn execute_command(
        &self,
        request: Request<ExecuteCommandRequest>,
    ) -> Result<Response<DataframeResponse>, Status> {
        let start_time = Instant::now();
        let req = request.into_inner();
        let command: cli::Commands = match serde_json::from_str(&req.command_json) {
            Ok(cmd) => cmd,
            Err(e) => {
                return Ok(Response::new(DataframeResponse {
                    dataframe_json: "".to_string(),
                    success: false,
                    error: format!("Failed to deserialize command: {}", e),
                }));
            }
        };
        let result = command.run();
        let duration = start_time.elapsed();
        match result {
            Ok(df) => {
                let df_json = match serde_json::to_string(&df) {
                    Ok(json) => json,
                    Err(e) => {
                        return Ok(Response::new(DataframeResponse {
                            dataframe_json: "".to_string(),
                            success: false,
                            error: format!("Failed to serialize DataFrame: {}", e),
                        }));
                    }
                };
                histogram!(
                    "rmap_execute_command_duration_ms",
                    duration.as_millis() as f64
                );

                match command {
                    cli::Commands::Generate { count, unique } => {
                        info!(
                            "Generate command completed: {} addresses, unique: {}",
                            count, unique
                        );
                    }
                    cli::Commands::Scan {
                        scan_type, target, ..
                    } => {
                        info!(
                            "Scan command completed: type {:?}, target: {:?}",
                            scan_type, target
                        );
                    }
                    cli::Commands::Discover => {
                        info!("Discover command completed");
                    }
                    cli::Commands::View { file, .. } => {
                        info!("View command completed: file {:?}", file);
                    }
                    cli::Commands::Analyze { file, analysis, .. } => {
                        info!(
                            "Analyze command completed: file {:?}, analysis: {:?}",
                            file, analysis
                        );
                    }
                    cli::Commands::Train => {
                        info!("Train command completed");
                    }
                    cli::Commands::Serve { .. } => {
                    }
                }

                Ok(Response::new(DataframeResponse {
                    dataframe_json: df_json,
                    success: true,
                    error: "".to_string(),
                }))
            }
            Err(e) => Ok(Response::new(DataframeResponse {
                dataframe_json: "".to_string(),
                success: false,
                error: e,
            })),
        }
    }
}

pub async fn run_server(
    addr: &str,
    metrics_port: Option<u16>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr.parse()?;
    let service = RmapServiceImpl::new();
    let metrics_port = metrics_port.unwrap_or(9090);
    if metrics_port == 0 {
        println!("Metrics disabled (port 0 specified)");
    } else {
        match metrics_exporter_prometheus::PrometheusBuilder::new()
            .with_http_listener(([0, 0, 0, 0], metrics_port))
            .install()
        {
            Ok(_) => {
                println!(
                    "Prometheus metrics available at http://0.0.0.0:{}/metrics",
                    metrics_port
                );
                counter!("rmap_server_starts_total", 1);
                gauge!("rmap_server_up", 1.0);
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to install Prometheus metrics exporter on port {}: {}",
                    metrics_port, e
                );
                eprintln!(
                    "Metrics will not be available. Try a different port or ensure port {} is not in use.",
                    metrics_port
                );
                eprintln!("You can disable metrics by using --metrics-port 0");
            }
        }
    }
    println!("Starting gRPC server on {}", addr);
    Server::builder()
        .add_service(RmapServiceServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}

pub async fn execute_remote_command(
    server_addr: &str,
    command: &cli::Commands,
) -> Result<DataFrame, Box<dyn std::error::Error>> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("|/-\\")
    );
    pb.set_message("Connecting to server...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut client = GrpcClient::new(server_addr.to_string()).await?;

    pb.set_message("Executing command...");
    let command_json = serde_json::to_string(command)?;
    let request = ExecuteCommandRequest { command_json };
    let response = client.client.execute_command(request).await?;

    pb.finish_and_clear();

    let response = response.into_inner();
    if !response.success {
        return Err(response.error.into());
    }
    let df: DataFrame = serde_json::from_str(&response.dataframe_json)?;
    Ok(df)
}

pub struct GrpcClient {
    client: rmap::rmap_service_client::RmapServiceClient<tonic::transport::Channel>,
}

impl GrpcClient {
    pub async fn new(addr: String) -> Result<Self, tonic::transport::Error> {
        let url = if addr.starts_with("http://") || addr.starts_with("https://") {
            addr
        } else {
            format!("http://{}", addr)
        };

        let client = rmap::rmap_service_client::RmapServiceClient::connect(url).await?;

        Ok(GrpcClient { client })
    }
}
