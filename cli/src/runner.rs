use clap::{Subcommand, ValueEnum};
use std::path::PathBuf;
use std::net::IpAddr;
use ipnet::IpNet;
use serde::{Serialize, Deserialize};
use polars::prelude::*;
use analyze::analysis::{FilterAnalysis, CountAnalysis};
use tga::TGA;

#[derive(Clone, ValueEnum, Serialize, Deserialize)]
#[value(rename_all = "snake_case")]
pub enum ProbeModule {
    TcpSynScan,
    IcmpEchoScan,
    UdpScan,
}

#[derive(Clone, Copy, ValueEnum, Debug, Serialize, Deserialize)]
#[value(rename_all = "snake_case")]
pub enum ScanType {
    Icmpv4,
    Icmpv6,
    LinkLocal,
}

#[derive(Clone, ValueEnum, Serialize, Deserialize)]
#[value(rename_all = "snake_case")]
pub enum ReservedPredicate {
    Loopback,
    Unspecified,
    LinkLocal,
    UniqueLocal,
}

#[derive(Clone, ValueEnum, Serialize, Deserialize)]
#[value(rename_all = "snake_case")]
pub enum MulticastPredicate {
    Multicast,
    SolicitedNode,
}

#[derive(Clone, ValueEnum, Serialize, Deserialize)]
#[value(rename_all = "snake_case")]
pub enum TransitionPredicate {
    Ipv4Mapped,
    Ipv4ToIpv6,
    ExtendedIpv4,
    Ipv6ToIpv4,
}

#[derive(Clone, ValueEnum, Serialize, Deserialize)]
#[value(rename_all = "snake_case")]
pub enum DocumentationPredicate {
    Documentation,
    Documentation2,
    Benchmarking,
}

#[derive(Clone, ValueEnum, Serialize, Deserialize)]
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

#[derive(Clone, ValueEnum, Serialize, Deserialize)]
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

#[derive(Clone, ValueEnum, Serialize, Deserialize)]
#[value(rename_all = "snake_case")]
pub enum Eui64Predicate {
    Eui64,
    LowByteHost,
}

#[derive(Subcommand, Serialize, Deserialize)]
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

#[derive(Subcommand, Serialize, Deserialize)]
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
        
        /// Prometheus metrics port (default: 9090, use 0 to disable)
        #[arg(short = 'm', long, default_value = "9090")]
        metrics_port: u16,
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

impl Commands {
    pub fn run(&self) -> Result<DataFrame, String> {
        match self {
            Commands::Generate { count, unique } => self.run_generate(*count, *unique),
            Commands::Scan { scan_type, target, .. } => self.run_scan(scan_type, target),
            Commands::Discover => self.run_discover(),
            Commands::Train => self.run_train(),
            Commands::View { file, field, reserved, multicast, transition, documentation, protocol, special_purpose, eui64, count, analysis, tui: _ } => {
                self.run_view(file, field, reserved, multicast, transition, documentation, protocol, special_purpose, eui64, *count, analysis)
            }
            Commands::Serve { .. } => {
                Err("Serve command cannot be executed remotely".to_string())
            }
        }
    }

    fn run_generate(&self, count: usize, unique: bool) -> Result<DataFrame, String> {
        let seed_ips: Vec<[u8; 16]> = vec![
            std::net::Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0001, 0, 0, 0, 0x0001).octets(),
            std::net::Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0001, 0, 0, 0, 0x0002).octets(),
            std::net::Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0002, 0, 0, 0, 0x0001).octets(),
            std::net::Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0002, 0, 0, 0, 0x0002).octets(),
            std::net::Ipv6Addr::new(0x2001, 0x0db8, 0x0002, 0x000a, 0, 0, 0, 0x000a).octets(),
            std::net::Ipv6Addr::new(0x2001, 0x0db8, 0x0002, 0x000a, 0, 0, 0, 0x000b).octets(),
            std::net::Ipv6Addr::new(0x2001, 0x0db8, 0x0002, 0x000b, 0, 0, 0, 0x000a).octets(),
            std::net::Ipv6Addr::new(0x2001, 0x0db8, 0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666).octets(),
            std::net::Ipv6Addr::new(0x2001, 0x0db8, 0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6667).octets(),
        ];
        let tga = match tga::EntropyIpTga::train(seed_ips) {
            Ok(tga) => tga,
            Err(e) => return Err(format!("Failed to train model: {}", e)),
        };
        let mut generated = std::collections::HashSet::new();
        let mut addresses = Vec::new();
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 1_000_000;
        while addresses.len() < count as usize {
            let generated_bytes = tga.generate();
            let generated_ip = std::net::Ipv6Addr::from(generated_bytes);
            if !unique || generated.insert(generated_ip) {
                addresses.push(generated_ip.to_string());
                attempts = 0;
            } else {
                attempts += 1;
                if attempts >= MAX_ATTEMPTS {
                    return Err(format!("Could only generate {}/{} unique addresses after {} attempts", addresses.len(), count, MAX_ATTEMPTS));
                }
            }
        }
        DataFrame::new(vec![Series::new("address".into(), addresses).into()])
            .map_err(|e| format!("Failed to create DataFrame: {}", e))
    }

    fn run_scan(&self, scan_type: &ScanType, target: &Option<String>) -> Result<DataFrame, String> {
        let target = match target {
            Some(t) => t,
            None => return Err("Target is required for non-link-local scans".to_string()),
        };
        let parsed_target = match super::Target::parse(target) {
            Ok(t) => t,
            Err(e) => return Err(format!("Failed to parse target: {}", e)),
        };
        let results = match (scan_type, parsed_target) {
            (ScanType::Icmpv4, super::Target::Network(ipnet::IpNet::V4(net))) => {
                scan::icmp6::icmp4_scan(net)
            }
            (ScanType::Icmpv6, super::Target::Network(ipnet::IpNet::V6(net))) => {
                scan::icmp6::icmp6_scan(net)
            }
            (ScanType::LinkLocal, _) => {
                let hosts = scan::link_local::discover_all_ipv6_link_local()
                    .map_err(|e| format!("Discovery failed: {}", e))?;
                hosts.into_iter().map(|host| {
                    scan::icmp6::ProbeResult {
                        addr: std::net::IpAddr::V6(host),
                        rtt: std::time::Duration::from_millis(0),
                    }
                }).collect()
            }
            _ => return Err("Unsupported scan type and target combination".to_string()),
        };
        let addresses: Vec<String> = results.iter().map(|r| r.addr.to_string()).collect();
        let rtts: Vec<u64> = results.iter().map(|r| r.rtt.as_millis() as u64).collect();
        DataFrame::new(vec![
            Series::new("address".into(), addresses).into(),
            Series::new("rtt_ms".into(), rtts).into(),
        ]).map_err(|e| format!("Failed to create DataFrame: {}", e))
    }

    fn run_discover(&self) -> Result<DataFrame, String> {
        let hosts = scan::link_local::discover_all_ipv6_link_local()
            .map_err(|e| format!("Discovery failed: {}", e))?;
        let results: Vec<scan::icmp6::ProbeResult> = hosts.into_iter().map(|host| {
            scan::icmp6::ProbeResult {
                addr: std::net::IpAddr::V6(host),
                rtt: std::time::Duration::from_millis(0),
            }
        }).collect();
        let addresses: Vec<String> = results.iter().map(|r| r.addr.to_string()).collect();
        let rtts: Vec<u64> = results.iter().map(|r| r.rtt.as_millis() as u64).collect();
        DataFrame::new(vec![
            Series::new("address".into(), addresses).into(),
            Series::new("rtt_ms".into(), rtts).into(),
        ]).map_err(|e| format!("Failed to create DataFrame: {}", e))
    }

    fn run_train(&self) -> Result<DataFrame, String> {
        let message = "Training functionality not yet implemented".to_string();
        DataFrame::new(vec![Series::new("message".into(), vec![message]).into()])
            .map_err(|e| format!("Failed to create DataFrame: {}", e))
    }

    fn run_view(&self, file: &PathBuf, field: &Option<String>, reserved: &Option<ReservedPredicate>, multicast: &Option<MulticastPredicate>, transition: &Option<TransitionPredicate>, documentation: &Option<DocumentationPredicate>, protocol: &Option<ProtocolPredicate>, special_purpose: &Option<SpecialPurposePredicate>, eui64: &Option<Eui64Predicate>, count: bool, analysis: &Option<ViewAnalysisCommand>) -> Result<DataFrame, String> {
        let df = crate::source::load_file(file, field);
        let processed_df = if let Some(reserved) = reserved {
            let reserved_name = match reserved {
                ReservedPredicate::Loopback => "loopback",
                ReservedPredicate::Unspecified => "unspecified",
                ReservedPredicate::LinkLocal => "link_local",
                ReservedPredicate::UniqueLocal => "unique_local",
            }.to_string();
            let columns = df.get_columns();
            if columns.len() == 1 {
                let analyzer = FilterAnalysis::new(reserved_name.clone());
                analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
            } else {
                let analyzer = FilterAnalysis::new(reserved_name.clone());
                analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
            }
        } else if let Some(multicast) = multicast {
            let multicast_name = match multicast {
                MulticastPredicate::Multicast => "multicast",
                MulticastPredicate::SolicitedNode => "solicited_node",
            }.to_string();
            let columns = df.get_columns();
            if columns.len() == 1 {
                let analyzer = FilterAnalysis::new(multicast_name.clone());
                analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
            } else {
                let analyzer = FilterAnalysis::new(multicast_name.clone());
                analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
            }
        } else if let Some(transition) = transition {
            let transition_name = match transition {
                TransitionPredicate::Ipv4Mapped => "ipv4_mapped",
                TransitionPredicate::Ipv4ToIpv6 => "ipv4_to_ipv6",
                TransitionPredicate::ExtendedIpv4 => "extended_ipv4",
                TransitionPredicate::Ipv6ToIpv4 => "ipv6_to_ipv4",
            }.to_string();
            let columns = df.get_columns();
            if columns.len() == 1 {
                let analyzer = FilterAnalysis::new(transition_name.clone());
                analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
            } else {
                let analyzer = FilterAnalysis::new(transition_name.clone());
                analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
            }
        } else if let Some(documentation) = documentation {
            let documentation_name = match documentation {
                DocumentationPredicate::Documentation => "documentation",
                DocumentationPredicate::Documentation2 => "documentation2",
                DocumentationPredicate::Benchmarking => "benchmarking",
            }.to_string();
            let columns = df.get_columns();
            if columns.len() == 1 {
                let analyzer = FilterAnalysis::new(documentation_name.clone());
                analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
            } else {
                let analyzer = FilterAnalysis::new(documentation_name.clone());
                analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
            }
        } else if let Some(protocol) = protocol {
            let protocol_name = match protocol {
                ProtocolPredicate::Teredo => "teredo",
                ProtocolPredicate::IetfProtocol => "ietf_protocol",
                ProtocolPredicate::PortControl => "port_control",
                ProtocolPredicate::Turn => "turn",
                ProtocolPredicate::DnsSd => "dns_sd",
                ProtocolPredicate::Amt => "amt",
                ProtocolPredicate::SegmentRouting => "segment_routing",
            }.to_string();
            let columns = df.get_columns();
            if columns.len() == 1 {
                let analyzer = FilterAnalysis::new(protocol_name.clone());
                analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
            } else {
                let analyzer = FilterAnalysis::new(protocol_name.clone());
                analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
            }
        } else if let Some(special_purpose) = special_purpose {
            let special_purpose_name = match special_purpose {
                SpecialPurposePredicate::DiscardOnly => "discard_only",
                SpecialPurposePredicate::DummyPrefix => "dummy_prefix",
                SpecialPurposePredicate::As112V6 => "as112v6",
                SpecialPurposePredicate::DirectAs112 => "direct_as112",
                SpecialPurposePredicate::DeprecatedOrchid => "deprecated_orchid",
                SpecialPurposePredicate::OrchidV2 => "orchid_v2",
                SpecialPurposePredicate::DroneRemoteId => "drone_remote_id",
            }.to_string();
            let columns = df.get_columns();
            if columns.len() == 1 {
                let analyzer = FilterAnalysis::new(special_purpose_name.clone());
                analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
            } else {
                let analyzer = FilterAnalysis::new(special_purpose_name.clone());
                analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
            }
        } else if let Some(eui64) = eui64 {
            let eui64_name = match eui64 {
                Eui64Predicate::Eui64 => "eui64",
                Eui64Predicate::LowByteHost => "low_byte_host",
            }.to_string();
            let columns = df.get_columns();
            if columns.len() == 1 {
                let analyzer = FilterAnalysis::new(eui64_name.clone());
                analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
            } else {
                let analyzer = FilterAnalysis::new(eui64_name.clone());
                analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
            }
        } else {
            df
        };
        if count {
            let columns = processed_df.get_columns();
            if columns.len() == 1 {
                let analyzer = CountAnalysis::new(None);
                let output = analyzer.analyze(columns[0].as_series().unwrap()).unwrap();
                return Ok(output);
            } else {
                let analyzer = CountAnalysis::new(None);
                let output = analyzer.analyze(columns[0].as_series().unwrap()).unwrap();
                return Ok(output);
            }
        }
        if let Some(analysis_cmd) = analysis {
            let result = match analysis_cmd {
                ViewAnalysisCommand::Unique => {
                    crate::analyze::analyze(processed_df.clone(), crate::analyze::AnalysisType::Unique)
                },
                ViewAnalysisCommand::Dispersion => {
                    crate::analyze::analyze(processed_df.clone(), crate::analyze::AnalysisType::Dispersion)
                },
                ViewAnalysisCommand::Entropy { start_bit, end_bit } => {
                    if start_bit >= end_bit {
                        return Err("start_bit must be less than end_bit".to_string());
                    }
                    crate::analyze::analyze(processed_df.clone(), crate::analyze::AnalysisType::Entropy {
                        start_bit: *start_bit,
                        end_bit: *end_bit,
                    })
                },
                ViewAnalysisCommand::Subnets { max_subnets, prefix_length } => {
                    crate::analyze::analyze(processed_df.clone(), crate::analyze::AnalysisType::Subnets {
                        max_subnets: *max_subnets,
                        prefix_length: *prefix_length,
                    })
                },
            };
            match result {
                Ok(_) => return Ok(processed_df),
                Err(e) => return Err(e.to_string()),
            }
        }
        Ok(processed_df)
    }
} 