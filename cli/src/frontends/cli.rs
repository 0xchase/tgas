use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::net::IpAddr;
use ipnet::IpNet;

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

#[derive(Clone, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum ProbeModule {
    TcpSynScan,
    IcmpEchoScan,
    UdpScan,
}

#[derive(Clone, Copy, ValueEnum, Debug)]
#[value(rename_all = "snake_case")]
pub enum ScanType {
    Icmpv4,
    Icmpv6,
    LinkLocal,
}

#[derive(Clone, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum ReservedPredicate {
    Loopback,
    Unspecified,
    LinkLocal,
    UniqueLocal,
}

#[derive(Clone, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum MulticastPredicate {
    Multicast,
    SolicitedNode,
}

#[derive(Clone, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum TransitionPredicate {
    Ipv4Mapped,
    Ipv4ToIpv6,
    ExtendedIpv4,
    Ipv6ToIpv4,
}

#[derive(Clone, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum DocumentationPredicate {
    Documentation,
    Documentation2,
    Benchmarking,
}

#[derive(Clone, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum ProtocolPredicate {
    Teredo,
    IetfProtocol,
    PortControl,
    Turn,
    DnsSd,
    Amt,
    SegmentRouting,
}

#[derive(Clone, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum SpecialPurposePredicate {
    DiscardOnly,
    DummyPrefix,
    As112V6,
    DirectAs112,
    DeprecatedOrchid,
    OrchidV2,
    DroneRemoteId,
}

#[derive(Clone, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum Eui64Predicate {
    Eui64,
    LowByteHost,
}

#[derive(Subcommand)]
pub enum ViewAnalysisCommand {
    /// Basic address counts and statistics
    Unique,
    /// Address space dispersion metrics
    Dispersion,
    /// Information entropy analysis
    Entropy {
        /// Start bit position (0-127) for entropy calculation
        #[arg(short = 's', long, value_parser = clap::value_parser!(u8).range(0..=127), default_value_t = 0)]
        start_bit: u8,

        /// End bit position (1-128) for entropy calculation
        #[arg(short = 'e', long, value_parser = clap::value_parser!(u8).range(1..=128), default_value_t = 128)]
        end_bit: u8,
    },
    /// Subnet distribution analysis
    Subnets {
        /// Maximum number of subnets to show (default: 10)
        #[arg(short = 'n', long, value_parser = clap::value_parser!(usize), default_value_t = 10)]
        max_subnets: usize,

        /// CIDR prefix length (default: 64)
        #[arg(short = 'l', long, value_parser = clap::value_parser!(u8).range(1..=128), default_value_t = 64)]
        prefix_length: u8,
    },
}

#[derive(Subcommand)]
pub enum Commands {
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
    /// Start gRPC server for remote command execution
    Serve {
        /// Server address to bind to (default: 127.0.0.1:50051)
        #[arg(short = 'a', long, default_value = "127.0.0.1:50051")]
        addr: String,
    },
    /// View and analyze data in an interactive TUI
    View {
        /// Path to file containing data to view
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Column name to select from input data
        #[arg(short = 'f', long, value_name = "FIELD")]
        field: Option<String>,

        /// Filter by reserved address types
        #[arg(long, value_enum)]
        reserved: Option<ReservedPredicate>,

        /// Filter by multicast address types
        #[arg(long, value_enum)]
        multicast: Option<MulticastPredicate>,

        /// Filter by transition address types
        #[arg(long, value_enum)]
        transition: Option<TransitionPredicate>,

        /// Filter by documentation address types
        #[arg(long, value_enum)]
        documentation: Option<DocumentationPredicate>,

        /// Filter by protocol address types
        #[arg(long, value_enum)]
        protocol: Option<ProtocolPredicate>,

        /// Filter by special purpose address types
        #[arg(long, value_enum)]
        special_purpose: Option<SpecialPurposePredicate>,

        /// Filter by EUI-64 address types
        #[arg(long, value_enum)]
        eui64: Option<Eui64Predicate>,

        /// Output counts rather than full IP list
        #[arg(short = 'c', long)]
        count: bool,

        /// Analysis subcommand to run
        #[command(subcommand)]
        analysis: Option<ViewAnalysisCommand>,

        /// Show the resulting dataframe in an interactive TUI
        #[arg(long)]
        tui: bool,
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
pub enum Target {
    SingleIp(IpAddr),
    Network(IpNet),
    Hostname(String, Vec<IpAddr>),
}

impl Target {
    pub fn parse(input: &str) -> Result<Self, TargetError> {
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

/// Helper function to convert CLI command to remote execution arguments
pub fn command_to_remote_args(command: &Commands) -> (String, Vec<String>) {
    let command_name = match command {
        Commands::Generate { .. } => "generate",
        Commands::Scan { .. } => "scan",
        Commands::View { .. } => "view",
        Commands::Discover => "discover",
        Commands::Train => "train",
        Commands::Serve { .. } => {
            panic!("Cannot execute serve command remotely");
        }
    };

    let mut args = vec![command_name.to_string()];
    
    // Add command-specific arguments
    match command {
        Commands::Generate { count, unique } => {
            args.push("--count".to_string());
            args.push(count.to_string());
            if *unique {
                args.push("--unique".to_string());
            }
        }
        Commands::Scan { scan_type, target, .. } => {
            args.push("--scan-type".to_string());
            args.push(format!("{:?}", scan_type));
            if let Some(t) = target {
                args.push(t.clone());
            }
        }
        Commands::View { file, .. } => {
            args.push(file.to_string_lossy().to_string());
        }
        _ => {}
    }

    (command_name.to_string(), args)
} 