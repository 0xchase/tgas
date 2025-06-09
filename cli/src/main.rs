use clap::{Parser, Subcommand, ValueEnum};
use comfy_table::{CellAlignment, Row};
use comfy_table::{Table, ContentArrangement, modifiers::UTF8_ROUND_CORNERS, Attribute, Cell};
use std::path::PathBuf;
use std::net::{IpAddr, ToSocketAddrs};
use std::fs::File;
use ipnet::IpNet;
use hickory_resolver::AsyncResolver;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use polars::prelude::*;
use analyze::{analyze, AnalysisType, analyze_file};

mod analyze;

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
    /// Special IPv6 address block analysis
    Special {
        /// Path to file containing IPv6 addresses
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Column name to select from input data
        #[arg(short = 'f', long, value_name = "FIELD")]
        field: Option<String>,
    },
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze IPv6 addresses from a file
    Analyze {
        #[command(subcommand)]
        command: AnalysisCommand,
    },
    /// Discover new targets by scanning the address space
    Discover,
    /// Generate a set of targets
    Generate {
        /// Number of addresses to generate
        #[arg(short = 'n', long, default_value = "10")]
        count: usize,

        /// Ensure generated addresses are unique
        #[arg(short = 'u', long)]
        unique: bool,
    },
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
    /// Train the TGA
    Train,
    /// View data in an interactive TUI
    View {
        /// Path to file containing data to view
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Column name to select from input data
        #[arg(short = 'f', long, value_name = "FIELD")]
        field: Option<String>,
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

fn format_cell(val: &polars::prelude::AnyValue) -> Cell {
    match val {
        AnyValue::Int64(_) | AnyValue::Int32(_) | AnyValue::Int16(_) | AnyValue::Int8(_) |
        AnyValue::UInt64(_) | AnyValue::UInt32(_) | AnyValue::UInt16(_) | AnyValue::UInt8(_) |
        AnyValue::Float64(_) | AnyValue::Float32(_) => {
            Cell::new(val.to_string())
                .set_alignment(CellAlignment::Right)
        },
        AnyValue::String(s) => {
            Cell::new(s.to_string())
        }
        _ => {
            Cell::new(val.to_string())
        }
    }
}

fn print_dataframe(df: &DataFrame) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.load_preset("     ──            ");
    
    // Add headers
    let headers: Vec<Cell> = df
        .get_column_names()
        .iter()
        .map(|s| Cell::new(s)
            .add_attribute(Attribute::Bold))
        .collect();
    table.set_header(headers);

    // Add data rows
    for i in 0..df.height() {
        let row = df.get_row(i).unwrap();
        let row_data: Vec<Cell> = row.0
            .iter()
            .map(|val| format_cell(val))
            .collect();

        table.add_row(row_data);
    }

    println!("\n");
    println!("{}", table);
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
                    analyze_file(&file, field, AnalysisType::Counts).await
                },
                AnalysisCommand::Dispersion { file, field } => {
                    analyze_file(&file, field, AnalysisType::Dispersion).await
                },
                AnalysisCommand::Entropy { file, field, start_bit, end_bit } => {
                    if start_bit >= end_bit {
                        eprintln!("Error: start_bit must be less than end_bit");
                        std::process::exit(1);
                    }
                    analyze_file(&file, field, AnalysisType::Entropy {
                        start_bit: *start_bit,
                        end_bit: *end_bit,
                    }).await
                },
                AnalysisCommand::Subnets { file, field, max_subnets, prefix_length } => {
                    analyze_file(&file, field, AnalysisType::Subnets {
                        max_subnets: *max_subnets,
                        prefix_length: *prefix_length,
                    }).await
                },
                AnalysisCommand::Special { file, field } => {
                    analyze_file(&file, field, AnalysisType::Special).await
                },
            };

            match result {
                Ok(df) => print_dataframe(&df),
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::View { file, field } => {
            // Try reading as CSV first
            let df = match CsvReader::new(File::open(&file)
                .map_err(|e| format!("Failed to open file: {}", e)).unwrap())
                .finish() {
                    Ok(df) => df,
                    Err(e) => {
                        // If CSV fails, try Parquet
                        ParquetReader::new(File::open(&file)
                            .map_err(|e| format!("Failed to open file: {}", e)).unwrap())
                            .finish()
                            .map_err(|e| format!("Failed to parse Parquet file: {}", e))
                            .map_err(|e| {
                                eprintln!("Error: {}", e);
                                std::process::exit(1);
                            }).unwrap()
                    }
                };
            
            let df = match field {
                Some(field) => df
                    .lazy()
                    .select([col(field)]),
                None => df
                    .lazy()
            };

            if let Err(e) = view::run_tui(df) {
                eprintln!("Error running TUI: {}", e);
                std::process::exit(1);
            }
        }
    }
}
