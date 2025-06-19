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
use analyze::{analyze, AnalysisType};
use sink::print_dataframe;

mod analyze;
mod source;
mod sink;

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

#[derive(Clone, Copy, ValueEnum)]
#[value(rename_all = "snake_case")]
enum ScanType {
    Icmpv4,
    Icmpv6,
    LinkLocal,
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
    /// EUI-64 address analysis (extract MAC addresses)
    Eui64 {
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
    /// Count addresses matching each predicate
    Count {
        /// Path to file containing IPv6 addresses
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Column name to select from input data
        #[arg(short = 'f', long, value_name = "FIELD")]
        field: Option<String>,

        /// Specific predicate name to count (default: all predicates)
        #[arg(short = 'p', long, value_name = "PREDICATE")]
        predicate: Option<String>,
    },
    /// Filter addresses by predicate
    Filter {
        /// Path to file containing IPv6 addresses
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Column name to select from input data
        #[arg(short = 'f', long, value_name = "FIELD")]
        field: Option<String>,

        /// Predicate name to filter by
        #[arg(short = 'p', long, value_name = "PREDICATE")]
        predicate: String,
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
        /// Type of scan to perform
        #[arg(short = 's', long, value_enum, default_value = "icmpv4")]
        scan_type: ScanType,

        /// Target specification (IP, hostname, or CIDR range) - not needed for link-local scans
        #[arg(value_name = "TARGET")]
        target: Option<String>,

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
        #[arg(short = 'o', long)]
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
    fn parse(input: &str) -> Result<Self, TargetError> {
        // Try parsing as IP address first
        if let Ok(ip) = input.parse::<IpAddr>() {
            return Ok(Target::SingleIp(ip));
        }

        // Try parsing as CIDR network
        if let Ok(net) = input.parse::<IpNet>() {
            return Ok(Target::Network(net));
        }

        // Try resolving as hostname
        /*let resolver = AsyncResolver::tokio(
            ResolverConfig::default(),
            ResolverOpts::default(),
        );
        
        let response = resolver.lookup_ip(input).await
            .map_err(TargetError::DnsResolve)?;
            
        let addresses: Vec<IpAddr> = response.iter().collect();
        
        if addresses.is_empty() {
            return Err(TargetError::NoAddressFound);
        }

        Ok(Target::Hostname(input.to_string(), addresses))*/
        todo!()
    }
}

fn main() {
    let cli = Cli::parse();

    if let Some(log_path) = &cli.log {
        println!("Logging to file: {:?}", log_path);
    }

    match &cli.command {
        Commands::Count { file, field, predicate } => {
            let df = source::load_file(file, field);
            for column in df.get_columns() {
                let analyzer = ::analyze::analysis::CountAnalysis::new(predicate.clone());
                let output = analyzer.analyze(column.as_series().unwrap()).unwrap();
                print_dataframe(&output);
            }
        },
        Commands::Filter { file, field, predicate } => {
            let df = source::load_file(file, field);
            for column in df.get_columns() {
                let analyzer = ::analyze::analysis::FilterAnalysis::new(predicate.clone());
                let output = analyzer.analyze(column.as_series().unwrap()).unwrap();
                print_dataframe(&output);
            }
        },
        Commands::Generate { count, unique } => {
            println!("Generating {} addresses{}", count, if *unique { " (unique)" } else { "" });
            tga::generate(*count, *unique);
        },
        Commands::Train => {
            println!("Running 'train' command");
            // TODO: implement train logic
        },
        Commands::Scan { 
            scan_type,
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
            match scan_type {
                ScanType::LinkLocal => {
                    println!("Performing IPv6 link-local discovery scan...");
                    match scan::link_local::discover_all_ipv6_link_local() {
                        Ok(hosts) => {
                            println!("Found {} IPv6 hosts:", hosts.len());
                            for host in hosts {
                                println!("  - {}", host);
                            }
                        }
                        Err(e) => {
                            eprintln!("Link-local discovery failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                ScanType::Icmpv4 | ScanType::Icmpv6 => {
                    // Parse the target for ICMP scans
                    let target_str = match target {
                        Some(t) => t,
                        None => {
                            eprintln!("Error: Target is required for ICMP scans");
                            std::process::exit(1);
                        }
                    };
                    
                    match Target::parse(&target_str) {
                        Ok(Target::SingleIp(ip)) => {
                            println!("Targeting single IP: {}", ip);
                            // TODO: Implement single IP scanning
                        }
                        Ok(Target::Network(net)) => {
                            match (*scan_type, net) {
                                (ScanType::Icmpv4, IpNet::V4(net)) => {
                                    println!("Performing ICMPv4 scan of network: {}", net);
                                    let results = scan::icmp6::icmp4_scan(net);
                                    println!("Scan complete. Found {} responsive hosts:", results.len());
                                    for result in results {
                                        println!("  - {} (RTT: {:?})", result.addr, result.rtt);
                                    }
                                }
                                (ScanType::Icmpv6, IpNet::V6(net)) => {
                                    println!("Performing ICMPv6 scan of network: {}", net);
                                    let results = scan::icmp6::icmp6_scan(net);
                                    println!("Scan complete. Found {} responsive hosts:", results.len());
                                    for result in results {
                                        println!("  - {} (RTT: {:?})", result.addr, result.rtt);
                                    }
                                }
                                (ScanType::Icmpv4, IpNet::V6(_)) => {
                                    eprintln!("Error: ICMPv4 scan requires IPv4 network, got IPv6");
                                    std::process::exit(1);
                                }
                                (ScanType::Icmpv6, IpNet::V4(_)) => {
                                    eprintln!("Error: ICMPv6 scan requires IPv6 network, got IPv4");
                                    std::process::exit(1);
                                }
                                (ScanType::LinkLocal, _) => {
                                    eprintln!("Error: Link-local scans don't use network targets");
                                    std::process::exit(1);
                                }
                            }
                        }
                        Ok(Target::Hostname(name, ips)) => {
                            println!("Targeting hostname: {} (resolved to {} addresses)", name, ips.len());
                            for ip in ips {
                                println!(" - {}", ip);
                            }
                            // TODO: Implement hostname scanning
                        }
                        Err(e) => {
                            eprintln!("Error parsing target: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::Discover => {
            println!("Running 'discover' command");
            // TODO: implement discover logic
        }
        Commands::Analyze { command } => {
            let result = match command {
                AnalysisCommand::Counts { file, field } => {
                    let df = source::load_file(file, field);
                    analyze(df, AnalysisType::Counts)
                },
                AnalysisCommand::Dispersion { file, field } => {
                    let df = source::load_file(file, field);
                    analyze(df, AnalysisType::Dispersion)
                },
                AnalysisCommand::Entropy { file, field, start_bit, end_bit } => {
                    if start_bit >= end_bit {
                        eprintln!("Error: start_bit must be less than end_bit");
                        std::process::exit(1);
                    }
                    let df = source::load_file(file, field);
                    analyze(df, AnalysisType::Entropy {
                        start_bit: *start_bit,
                        end_bit: *end_bit,
                    })
                },
                AnalysisCommand::Subnets { file, field, max_subnets, prefix_length } => {
                    let df = source::load_file(file, field);
                    analyze(df, AnalysisType::Subnets {
                        max_subnets: *max_subnets,
                        prefix_length: *prefix_length,
                    })
                },
                AnalysisCommand::Special { file, field } => {
                    let df = source::load_file(file, field);
                    analyze(df, AnalysisType::Special)
                },
                AnalysisCommand::Eui64 { file, field } => {
                    let df = source::load_file(file, field);
                    analyze(df, AnalysisType::Eui64)
                },
            };

            match result {
                Ok(df) => (),
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
