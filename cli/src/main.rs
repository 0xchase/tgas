use analyze::{AnalysisType, analyze};
use clap::Parser;
use comfy_table::{Attribute, Cell, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS};
use comfy_table::{CellAlignment, Row};
use hickory_resolver::AsyncResolver;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use ipnet::IpNet;
use polars::lazy::dsl::col;
use polars::prelude::*;
use sink::print_dataframe;
use std::fs::File;
use std::net::{IpAddr, ToSocketAddrs};
use std::path::PathBuf;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use indicatif::{ProgressState, ProgressStyle};
use std::time::Duration;
use time;
use tracing::{error, info, info_span};

mod analyze;
mod frontends;
mod runner;
mod sink;
mod source;

use frontends::cli::{Cli, Commands};
use frontends::grpc::{execute_remote_command, run_server};

fn elapsed_subsec(state: &ProgressState, writer: &mut dyn std::fmt::Write) {
    let elapsed = state.elapsed();
    let _ = write!(writer, "{:.1}s", elapsed.as_secs_f64());
}

fn main() {
    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_span_events(fmt::format::FmtSpan::NONE)
        .with_timer(fmt::time::LocalTime::new(
            time::macros::format_description!("[hour]:[minute]:[second]"),
        ));
    let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    let cli = Cli::parse();

    if let Some(log_path) = &cli.log {
        info!("Logging to file: {:?}", log_path);
    }

    if let Some(server_addr) = &cli.remote {
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(execute_remote_command(server_addr, &cli.command)) {
            Ok(df) => {
                print_dataframe(&df);
            }
            Err(e) => {
                error!("Remote execution failed: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    match &cli.command {
        Commands::Serve { addr, metrics_port } => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(e) = rt.block_on(frontends::grpc::run_server(addr, Some(*metrics_port))) {
                error!("Failed to start server: {}", e);
                std::process::exit(1);
            }
        }
        _ => match cli.command.run() {
            Ok(df) => print_dataframe(&df),
            Err(e) => {
                error!("Error: {}", e);
                std::process::exit(1);
            }
        },
    }
}
