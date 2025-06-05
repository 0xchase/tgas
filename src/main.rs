use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::net::Ipv6Addr;

mod scan;

/// A simple example of clap
#[derive(Parser)]
#[command(
    name = "main",
    about = "A simple example of clap",
    version = "0.1.0",
    author = "Your Name"
)]
struct Cli {
    /// Increase output verbosity
    #[arg(short, long)]
    verbose: bool,

    /// Log file path
    #[arg(short, long, value_name = "LOG_FILE")]
    log: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a set of targets
    Generate,
    /// Train the TGA
    Train,
    /// Scan the given address set
    Scan,
    /// Discover new targets by scanning the address space
    Discover,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Handle global flags
    if cli.verbose {
        println!("Verbose mode enabled");
    }
    if let Some(log_path) = &cli.log {
        println!("Logging to file: {:?}", log_path);
    }

    match &cli.command {
        Commands::Generate => {
            println!("Running 'generate' command");
            // TODO: implement generate logic
        }
        Commands::Train => {
            println!("Running 'train' command");
            // TODO: implement train logic
        }
        Commands::Scan => {
            scan::test_scan().await;
        }
        Commands::Discover => {
            println!("Running 'discover' command");
            // TODO: implement discover logic
        }
    }
}

trait Job {
    // Run the scan, tga, training, or whatever
    fn run();

    // Status of the asynchronously running job
    fn status() -> String;
}

// generates new targets given a seed
/*
Don't use static and dynamic TGAs
- Pure algorithmic methods are a simple TGA type
- Training a model may output a TGA
- Dynamic TGAs transform a TGA into another TGA
*/
trait TGA {}

// May analyze or visualize a scan result
trait Analyzer {}
