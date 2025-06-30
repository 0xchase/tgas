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

mod analyze;
mod source;
mod sink;
mod frontends;
mod runner;

use frontends::cli::{Cli, Commands, Target};
use frontends::grpc::{run_server, execute_remote_command};

fn main() {
    let cli = Cli::parse();

    if let Some(log_path) = &cli.log {
        println!("Logging to file: {:?}", log_path);
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
                eprintln!("Remote execution failed: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    match &cli.command {
        Commands::Serve { addr, metrics_port } => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(e) = rt.block_on(frontends::grpc::run_server(addr, Some(*metrics_port))) {
                eprintln!("Failed to start server: {}", e);
                std::process::exit(1);
            }
        }
        _ => {
            match cli.command.run() {
                Ok(df) => print_dataframe(&df),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
} 