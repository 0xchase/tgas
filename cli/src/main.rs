use clap::Parser;
use comfy_table::{CellAlignment, Row};
use comfy_table::{Table, ContentArrangement, modifiers::UTF8_ROUND_CORNERS, Attribute, Cell};
use std::path::PathBuf;
use std::net::{IpAddr, ToSocketAddrs};
use std::fs::File;
use ipnet::IpNet;
use hickory_resolver::AsyncResolver;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use polars::prelude::*;
use polars::lazy::dsl::col;
use analyze::{analyze, AnalysisType};
use sink::print_dataframe;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing_indicatif::IndicatifLayer;

use tracing::{info, error, info_span};
use time;
use std::time::Duration;
use indicatif::{ProgressStyle, ProgressState};

mod analyze;
mod source;
mod sink;
mod frontends;
mod runner;

use frontends::cli::{Cli, Commands};
use frontends::grpc::{run_server, execute_remote_command};

fn elapsed_subsec(state: &ProgressState, writer: &mut dyn std::fmt::Write) {
    let elapsed = state.elapsed();
    let _ = write!(writer, "{:.1}s", elapsed.as_secs_f64());
}

fn main() {
    // Set up tracing with an indicatif progress bar layer
    //let indicatif_layer = IndicatifLayer::new();
    /*let indicatif_layer = IndicatifLayer::new().with_progress_style(
        ProgressStyle::with_template(
            "{color_start}Working... {wide_msg} [{bar:20.cyan/blue}] {pos}/{len} {elapsed_subsec}{color_end}",
        )
        .unwrap()
        .with_key(
            "elapsed_subsec",
            elapsed_subsec,
        )
        .with_key(
            "color_start",
            |state: &ProgressState, writer: &mut dyn std::fmt::Write| {
                let elapsed = state.elapsed();

                if elapsed > Duration::from_secs(8) {
                    // Red
                    let _ = write!(writer, "\x1b[31m");
                } else if elapsed > Duration::from_secs(4) {
                    // Yellow
                    let _ = write!(writer, "\x1b[33m");
                }
            },
        )
        .with_key(
            "color_end",
            |state: &ProgressState, writer: &mut dyn std::fmt::Write| {
                if state.elapsed() > Duration::from_secs(4) {
                    let _ =write!(writer, "\x1b[0m");
                }
            },
        ),
    ).with_span_child_prefix_symbol("↳ ").with_span_child_prefix_indent(" ");*/

    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_span_events(fmt::format::FmtSpan::NONE)
        .with_timer(fmt::time::LocalTime::new(
            time::macros::format_description!("[hour]:[minute]:[second]")
        ));
    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    let cli = Cli::parse();

    if let Some(log_path) = &cli.log {
        info!("Logging to file: {:?}", log_path);
    }

    // Handle remote execution if --remote flag is provided
    if let Some(server_addr) = &cli.remote {
        // Execute remotely by passing the command directly
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
        _ => {
            match cli.command.run() {
                Ok(df) => print_dataframe(&df),
                Err(e) => {
                    error!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
} 