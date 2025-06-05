use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::net::{IpAddr, ToSocketAddrs};
use ipnet::IpNet;
use hickory_resolver::AsyncResolver;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};

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
enum Commands {
    /// Generate a set of targets
    Generate {
        /// Number of addresses to generate
        #[arg(short = 'n', long, default_value = "10")]
        count: usize,
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Set up logging based on verbosity
    /*match cli.verbose {
        0 => println!("Running in quiet mode"),
        1 => println!("Running with normal verbosity"),
        2 => println!("Running with increased verbosity"),
        _ => println!("Running with maximum verbosity"),
    }*/

    if let Some(log_path) = &cli.log {
        println!("Logging to file: {:?}", log_path);
    }

    // println!("Output will be written to: {}", cli.output_file);

    match &cli.command {
        Commands::Generate { count } => {
            println!("Generating {} addresses...", count);
            // TODO: implement proper model building and generation
            tga::test(*count);
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
    }
}

trait Job {
    // Run the scan, tga, training, or whatever
    fn run();

    // Status of the asynchronously running job
    fn status() -> String;
}
