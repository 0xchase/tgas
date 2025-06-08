use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::net::{IpAddr, ToSocketAddrs};
use std::io::BufReader;
use std::fs::File;
use ipnet::IpNet;
use hickory_resolver::AsyncResolver;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use analyze::{AnalysisType};
use polars::prelude::*;

/// A simple example of clap
#[derive(Parser)]
#[command(
    name = "ipv6kit",
    about = "IPv6 network scanning and analysis toolkit",
    version = "0.1.0",
    author = "Chase Kanipe"
)]
struct Cli {
    /// Increase output verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Log file path
    #[arg(short, long, value_name = "LOG_FILE")]
    log: Option<PathBuf>,

    /// Output file (use "-" for stdout)
    #[arg(short = 'o', long, default_value = "-")]
    output_file: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, ValueEnum)]
#[value(rename_all = "snake_case")]
enum ProbeModule {
    TcpSynScan,
    IcmpEchoScan,
    UdpScan,
}

#[derive(Subcommand)]
enum AnalysisCommand {
    /// Basic address counts and statistics
    Counts {
        /// Path to file containing IPv6 addresses
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Column name to select from input data
        #[arg(short = 'f', long, value_name = "FIELD")]
        field: Option<String>,
    },
    /// Address space dispersion metrics
    Dispersion {
        /// Path to file containing IPv6 addresses
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Column name to select from input data
        #[arg(short = 'f', long, value_name = "FIELD")]
        field: Option<String>,
    },
    /// Information entropy analysis
    Entropy {
        /// Path to file containing IPv6 addresses
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Column name to select from input data
        #[arg(short = 'f', long, value_name = "FIELD")]
        field: Option<String>,

        /// Start bit position (0-127) for entropy calculation
        #[arg(short = 's', long, value_parser = clap::value_parser!(u8).range(0..=127), default_value_t = 0)]
        start_bit: u8,

        /// End bit position (1-128) for entropy calculation
        #[arg(short = 'e', long, value_parser = clap::value_parser!(u8).range(1..=128), default_value_t = 128)]
        end_bit: u8,
    },
    /// Subnet distribution analysis
    Subnets {
        /// Path to file containing IPv6 addresses
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Column name to select from input data
        #[arg(short = 'f', long, value_name = "FIELD")]
        field: Option<String>,
        
        /// Maximum number of subnets to show (default: 10)
        #[arg(short = 'n', long, value_parser = clap::value_parser!(usize), default_value_t = 10)]
        max_subnets: usize,

        /// CIDR prefix length (default: 64)
        #[arg(short = 'l', long, value_parser = clap::value_parser!(u8).range(1..=128), default_value_t = 64)]
        prefix_length: u8,
    },
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a set of targets
    Generate {
        /// Number of addresses to generate
        #[arg(short = 'n', long, default_value = "10")]
        count: usize,

        /// Ensure generated addresses are unique
        #[arg(short = 'u', long)]
        unique: bool,
    },
    /// Train the TGA
    Train,
    /// Scan the given address set
    Scan {
        /// Target specification (IP, hostname, or CIDR range)
        #[arg(value_name = "TARGET")]
        target: String,

        /// Target port(s) to scan. Can be a single port, comma-separated list, or range (e.g. 80,443,8000-8010)
        #[arg(short = 'p', long, value_name = "PORT(S)")]
        target_ports: Option<String>,

        /// Input file containing targets (one per line)
        #[arg(short = 'I', long)]
        input_file: Option<PathBuf>,

        /// File containing CIDR ranges to exclude
        #[arg(short = 'b', long)]
        blocklist_file: Option<PathBuf>,

        /// File containing CIDR ranges to include
        #[arg(short = 'w', long)]
        allowlist_file: Option<PathBuf>,

        /// Maximum number of targets to probe
        #[arg(short = 'n', long)]
        max_targets: Option<String>,

        /// Send rate in packets per second
        #[arg(short = 'r', long, default_value = "10000")]
        rate: u32,

        /// Bandwidth cap (e.g. 10M, 1G)
        #[arg(short = 'B', long)]
        bandwidth: Option<String>,

        /// Number of probes to send to each target
        #[arg(short = 'P', long, default_value = "1")]
        probes: u32,

        /// Maximum runtime in seconds
        #[arg(short = 't', long)]
        max_runtime: Option<u32>,

        /// Cooldown time in seconds
        #[arg(short = 'c', long, default_value = "8")]
        cooldown_time: u32,

        /// Random seed for target selection
        #[arg(short = 'e', long)]
        seed: Option<u64>,

        /// Source port(s) to use
        #[arg(short = 's', long)]
        source_port: Option<String>,

        /// Source IP address(es) to use
        #[arg(short = 'S', long)]
        source_ip: Option<String>,

        /// Network interface to use
        #[arg(short = 'i', long)]
        interface: Option<String>,

        /// Type of probe to send
        #[arg(short = 'M', long, value_enum, default_value = "tcp_syn_scan")]
        probe_module: ProbeModule,

        /// Run in dry-run mode (print packets instead of sending)
        #[arg(short = 'd', long)]
        dryrun: bool,
    },
    /// Discover new targets by scanning the address space
    Discover,
    /// Analyze IPv6 addresses from a file
    Analyze {
        #[command(subcommand)]
        command: AnalysisCommand,
    },
}

#[derive(Debug)]
pub enum TargetError {
    IpAddrParse(std::net::AddrParseError),
    IpNetParse(ipnet::AddrParseError),
    DnsResolve(hickory_resolver::error::ResolveError),
    NoAddressFound,
}

impl std::fmt::Display for TargetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetError::IpAddrParse(e) => write!(f, "Failed to parse IP address: {}", e),
            TargetError::IpNetParse(e) => write!(f, "Failed to parse IP network: {}", e),
            TargetError::DnsResolve(e) => write!(f, "Failed to resolve hostname: {}", e),
            TargetError::NoAddressFound => write!(f, "No valid IP addresses found for hostname"),
        }
    }
}

#[derive(Debug)]
enum Target {
    SingleIp(IpAddr),
    Network(IpNet),
    Hostname(String, Vec<IpAddr>),
}

impl Target {
    async fn parse(input: &str) -> Result<Self, TargetError> {
        // Try parsing as IP address first
        if let Ok(ip) = input.parse::<IpAddr>() {
            return Ok(Target::SingleIp(ip));
        }

        // Try parsing as CIDR network
        if let Ok(net) = input.parse::<IpNet>() {
            return Ok(Target::Network(net));
        }

        // Try resolving as hostname
        let resolver = AsyncResolver::tokio(
            ResolverConfig::default(),
            ResolverOpts::default(),
        );
        
        let response = resolver.lookup_ip(input).await
            .map_err(TargetError::DnsResolve)?;
            
        let addresses: Vec<IpAddr> = response.iter().collect();
        
        if addresses.is_empty() {
            return Err(TargetError::NoAddressFound);
        }

        Ok(Target::Hostname(input.to_string(), addresses))
    }
}

fn analyze_file(
    file: &PathBuf,
    field: Option<&str>,
    analysis_type: analyze::AnalysisType,
) -> Result<(), String> {
    // Try reading as CSV first
    let df = CsvReader::new(File::open(file)
        .map_err(|e| format!("Failed to open file: {}", e))?)
        .finish()
        .map_err(|e| format!("Failed to parse CSV file: {}", e))
        .or_else(|_| {
            // If CSV fails, try Parquet
            ParquetReader::new(File::open(file)
                .map_err(|e| format!("Failed to open file: {}", e))?)
                .finish()
                .map_err(|e| format!("Failed to parse Parquet file: {}", e))
        })?;
    
    let df = match field {
        Some(field) => df
            .lazy()
            .select([col(field)]),
        None => df
            .lazy()
    };

    match analyze::analyze(df, analysis_type) {
        Ok(results) => {
            println!("{}", results);
            Ok(())
        },
        Err(e) => Err(format!("Analysis failed: {}", e)),
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Some(log_path) = &cli.log {
        println!("Logging to file: {:?}", log_path);
    }

    match &cli.command {
        Commands::Generate { count, unique } => {
            println!("Generating {} addresses{}", count, if *unique { " (unique)" } else { "" });
            tga::generate(*count, *unique);
        }
        Commands::Train => {
            println!("Running 'train' command");
            // TODO: implement train logic
        }
        Commands::Scan { 
            target,
            target_ports,
            input_file,
            blocklist_file,
            allowlist_file,
            max_targets,
            rate,
            bandwidth,
            probes,
            max_runtime,
            cooldown_time,
            seed,
            source_port,
            source_ip,
            interface,
            probe_module,
            dryrun,
        } => {
            // Parse the target
            match Target::parse(target).await {
                Ok(Target::SingleIp(ip)) => {
                    println!("Targeting single IP: {}", ip);
                }
                Ok(Target::Network(net)) => {
                    println!("Targeting network: {}", net);
                }
                Ok(Target::Hostname(name, ips)) => {
                    println!("Targeting hostname: {} (resolved to {} addresses)", name, ips.len());
                    for ip in ips {
                        println!(" - {}", ip);
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing target: {}", e);
                    std::process::exit(1);
                }
            }

            /*println!("Configuring scan with:");
            if let Some(ports) = target_ports {
                println!("  Ports: {}", ports);
            }
            println!("  Rate: {} pps", rate);
            if *dryrun {
                println!("Running in dry-run mode");
            }*/

            // TODO: Configure scanner with these parameters
            scan::test_scan().await;
        }
        Commands::Discover => {
            println!("Running 'discover' command");
            // TODO: implement discover logic
        }
        Commands::Analyze { command } => {
            let result = match command {
                AnalysisCommand::Counts { file, field } => {
                    analyze_file(file, field.as_deref(), analyze::AnalysisType::Counts)
                },
                AnalysisCommand::Dispersion { file, field } => {
                    analyze_file(file, field.as_deref(), analyze::AnalysisType::Dispersion)
                },
                AnalysisCommand::Entropy { file, field, start_bit, end_bit } => {
                    if start_bit >= end_bit {
                        eprintln!("Error: start_bit must be less than end_bit");
                        std::process::exit(1);
                    }
                    analyze_file(file, field.as_deref(), analyze::AnalysisType::Entropy {
                        start_bit: *start_bit,
                        end_bit: *end_bit,
                    })
                },
                AnalysisCommand::Subnets { file, field, max_subnets, prefix_length } => {
                    analyze_file(file, field.as_deref(), analyze::AnalysisType::Subnets {
                        max_subnets: *max_subnets,
                        prefix_length: *prefix_length,
                    })
                },
            };

            if let Err(e) = result {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }
}

trait Job {
    // Run the scan, tga, training, or whatever
    fn run();

    // Status of the asynchronously running job
    fn status() -> String;
}
