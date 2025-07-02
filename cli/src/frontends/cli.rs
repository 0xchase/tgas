use clap::Parser;
use std::path::PathBuf;

pub use crate::runner::Commands;

/// CLI argument parser for the IPv6 toolkit
#[derive(Parser)]
#[command(
    name = "ipv6kit",
    about = "IPv6 network scanning and analysis toolkit",
    version = "0.1.0",
    author = "Chase Kanipe"
)]
pub struct Cli {
    /// Increase output verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Log file path
    #[arg(short, long, value_name = "LOG_FILE")]
    pub log: Option<PathBuf>,

    /// Output file (use "-" for stdout)
    #[arg(short = 'o', long, default_value = "-")]
    pub output_file: String,

    /// Remote server address for command execution (e.g., 127.0.0.1:50051)
    #[arg(long, value_name = "SERVER_ADDR")]
    pub remote: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}
