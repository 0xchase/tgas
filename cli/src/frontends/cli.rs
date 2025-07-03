use clap::Parser;
use std::path::PathBuf;

pub use crate::runner::Commands;

#[derive(Parser)]
#[command(
    name = "rmap",
    about = "IPv6 network scanning and analysis toolkit",
    version = "0.1.0",
    author = "Chase Kanipe"
)]
pub struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[arg(short, long, value_name = "LOG_FILE")]
    pub log: Option<PathBuf>,

    #[arg(short = 'o', long, default_value = "-")]
    pub output_file: String,

    #[arg(long, value_name = "SERVER_ADDR")]
    pub remote: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}
