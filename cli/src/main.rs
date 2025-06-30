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

use frontends::cli::{Cli, Commands, Target, command_to_remote_args};
use frontends::grpc::{run_server, execute_remote_command};

fn main() {
    let cli = Cli::parse();

    if let Some(log_path) = &cli.log {
        println!("Logging to file: {:?}", log_path);
    }

    // Handle remote execution if --remote flag is provided
    if let Some(server_addr) = &cli.remote {
        // Convert command to arguments for remote execution
        let (command, args) = command_to_remote_args(&cli.command);

        // Execute remotely
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Err(e) = rt.block_on(execute_remote_command(server_addr, &command, args)) {
            eprintln!("Remote execution failed: {}", e);
            std::process::exit(1);
        }
        return;
    }

    match &cli.command {
        Commands::Generate { count, unique } => {
            println!("Generating {} addresses{}", count, if *unique { " (unique)" } else { "" });
            tga::generate(*count, *unique);
        },
        Commands::Train => {
            println!("Running 'train' command");
            // TODO: implement train logic
        },
        Commands::Serve { addr } => {
            println!("Starting gRPC server on {}", addr);
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(e) = rt.block_on(run_server(addr)) {
                eprintln!("Server failed: {}", e);
                std::process::exit(1);
            }
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
                frontends::cli::ScanType::LinkLocal => {
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
                frontends::cli::ScanType::Icmpv4 | frontends::cli::ScanType::Icmpv6 => {
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
                                (frontends::cli::ScanType::Icmpv4, IpNet::V4(net)) => {
                                    println!("Performing ICMPv4 scan of network: {}", net);
                                    let results = scan::icmp6::icmp4_scan(net);
                                    println!("Scan complete. Found {} responsive hosts:", results.len());
                                    for result in results {
                                        println!("  - {} (RTT: {:?})", result.addr, result.rtt);
                                    }
                                }
                                (frontends::cli::ScanType::Icmpv6, IpNet::V6(net)) => {
                                    println!("Performing ICMPv6 scan of network: {}", net);
                                    let results = scan::icmp6::icmp6_scan(net);
                                    println!("Scan complete. Found {} responsive hosts:", results.len());
                                    for result in results {
                                        println!("  - {} (RTT: {:?})", result.addr, result.rtt);
                                    }
                                }
                                (frontends::cli::ScanType::Icmpv4, IpNet::V6(_)) => {
                                    eprintln!("Error: ICMPv4 scan requires IPv4 network, got IPv6");
                                    std::process::exit(1);
                                }
                                (frontends::cli::ScanType::Icmpv6, IpNet::V4(_)) => {
                                    eprintln!("Error: ICMPv6 scan requires IPv6 network, got IPv4");
                                    std::process::exit(1);
                                }
                                (frontends::cli::ScanType::LinkLocal, _) => {
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
        Commands::View { file, field, reserved, multicast, transition, documentation, protocol, special_purpose, eui64, count, analysis, tui } => {
            // Load the data file
            let df = source::load_file(file, field);
            
            // Apply filtering if specified
            let processed_df = if let Some(reserved) = reserved {
                let reserved_name = match reserved {
                    frontends::cli::ReservedPredicate::Loopback => "loopback",
                    frontends::cli::ReservedPredicate::Unspecified => "unspecified",
                    frontends::cli::ReservedPredicate::LinkLocal => "link_local",
                    frontends::cli::ReservedPredicate::UniqueLocal => "unique_local",
                }.to_string();
                let columns = df.get_columns();
                if columns.len() == 1 {
                    let analyzer = ::analyze::analysis::FilterAnalysis::new(reserved_name.clone());
                    analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
                } else {
                    // For multiple columns, just use the first one for filtering
                    let analyzer = ::analyze::analysis::FilterAnalysis::new(reserved_name.clone());
                    analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
                }
            } else if let Some(multicast) = multicast {
                let multicast_name = match multicast {
                    frontends::cli::MulticastPredicate::Multicast => "multicast",
                    frontends::cli::MulticastPredicate::SolicitedNode => "solicited_node",
                }.to_string();
                let columns = df.get_columns();
                if columns.len() == 1 {
                    let analyzer = ::analyze::analysis::FilterAnalysis::new(multicast_name.clone());
                    analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
                } else {
                    // For multiple columns, just use the first one for filtering
                    let analyzer = ::analyze::analysis::FilterAnalysis::new(multicast_name.clone());
                    analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
                }
            } else if let Some(transition) = transition {
                let transition_name = match transition {
                    frontends::cli::TransitionPredicate::Ipv4Mapped => "ipv4_mapped",
                    frontends::cli::TransitionPredicate::Ipv4ToIpv6 => "ipv4_to_ipv6",
                    frontends::cli::TransitionPredicate::ExtendedIpv4 => "extended_ipv4",
                    frontends::cli::TransitionPredicate::Ipv6ToIpv4 => "ipv6_to_ipv4",
                }.to_string();
                let columns = df.get_columns();
                if columns.len() == 1 {
                    let analyzer = ::analyze::analysis::FilterAnalysis::new(transition_name.clone());
                    analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
                } else {
                    // For multiple columns, just use the first one for filtering
                    let analyzer = ::analyze::analysis::FilterAnalysis::new(transition_name.clone());
                    analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
                }
            } else if let Some(documentation) = documentation {
                let documentation_name = match documentation {
                    frontends::cli::DocumentationPredicate::Documentation => "documentation",
                    frontends::cli::DocumentationPredicate::Documentation2 => "documentation2",
                    frontends::cli::DocumentationPredicate::Benchmarking => "benchmarking",
                }.to_string();
                let columns = df.get_columns();
                if columns.len() == 1 {
                    let analyzer = ::analyze::analysis::FilterAnalysis::new(documentation_name.clone());
                    analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
                } else {
                    // For multiple columns, just use the first one for filtering
                    let analyzer = ::analyze::analysis::FilterAnalysis::new(documentation_name.clone());
                    analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
                }
            } else if let Some(protocol) = protocol {
                let protocol_name = match protocol {
                    frontends::cli::ProtocolPredicate::Teredo => "teredo",
                    frontends::cli::ProtocolPredicate::IetfProtocol => "ietf_protocol",
                    frontends::cli::ProtocolPredicate::PortControl => "port_control",
                    frontends::cli::ProtocolPredicate::Turn => "turn",
                    frontends::cli::ProtocolPredicate::DnsSd => "dns_sd",
                    frontends::cli::ProtocolPredicate::Amt => "amt",
                    frontends::cli::ProtocolPredicate::SegmentRouting => "segment_routing",
                }.to_string();
                let columns = df.get_columns();
                if columns.len() == 1 {
                    let analyzer = ::analyze::analysis::FilterAnalysis::new(protocol_name.clone());
                    analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
                } else {
                    // For multiple columns, just use the first one for filtering
                    let analyzer = ::analyze::analysis::FilterAnalysis::new(protocol_name.clone());
                    analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
                }
            } else if let Some(special_purpose) = special_purpose {
                let special_purpose_name = match special_purpose {
                    frontends::cli::SpecialPurposePredicate::DiscardOnly => "discard_only",
                    frontends::cli::SpecialPurposePredicate::DummyPrefix => "dummy_prefix",
                    frontends::cli::SpecialPurposePredicate::As112V6 => "as112v6",
                    frontends::cli::SpecialPurposePredicate::DirectAs112 => "direct_as112",
                    frontends::cli::SpecialPurposePredicate::DeprecatedOrchid => "deprecated_orchid",
                    frontends::cli::SpecialPurposePredicate::OrchidV2 => "orchid_v2",
                    frontends::cli::SpecialPurposePredicate::DroneRemoteId => "drone_remote_id",
                }.to_string();
                let columns = df.get_columns();
                if columns.len() == 1 {
                    let analyzer = ::analyze::analysis::FilterAnalysis::new(special_purpose_name.clone());
                    analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
                } else {
                    // For multiple columns, just use the first one for filtering
                    let analyzer = ::analyze::analysis::FilterAnalysis::new(special_purpose_name.clone());
                    analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
                }
            } else if let Some(eui64) = eui64 {
                let eui64_name = match eui64 {
                    frontends::cli::Eui64Predicate::Eui64 => "eui64",
                    frontends::cli::Eui64Predicate::LowByteHost => "low_byte_host",
                }.to_string();
                let columns = df.get_columns();
                if columns.len() == 1 {
                    let analyzer = ::analyze::analysis::FilterAnalysis::new(eui64_name.clone());
                    analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
                } else {
                    // For multiple columns, just use the first one for filtering
                    let analyzer = ::analyze::analysis::FilterAnalysis::new(eui64_name.clone());
                    analyzer.analyze(columns[0].as_series().unwrap()).unwrap()
                }
            } else {
                df
            };
            
            // Apply counting if specified
            if *count {
                let columns = processed_df.get_columns();
                if columns.len() == 1 {
                    let analyzer = ::analyze::analysis::CountAnalysis::new(None);
                    let output = analyzer.analyze(columns[0].as_series().unwrap()).unwrap();
                    print_dataframe(&output);
                } else {
                    // For multiple columns, just use the first one for counting
                    let analyzer = ::analyze::analysis::CountAnalysis::new(None);
                    let output = analyzer.analyze(columns[0].as_series().unwrap()).unwrap();
                    print_dataframe(&output);
                }
                return;
            }
            
            // Run analysis if specified
            if let Some(analysis_cmd) = analysis {
                let result = match analysis_cmd {
                    frontends::cli::ViewAnalysisCommand::Unique => {
                        analyze(processed_df, AnalysisType::Unique)
                    },
                    frontends::cli::ViewAnalysisCommand::Dispersion => {
                        analyze(processed_df, AnalysisType::Dispersion)
                    },
                    frontends::cli::ViewAnalysisCommand::Entropy { start_bit, end_bit } => {
                        if start_bit >= end_bit {
                            eprintln!("Error: start_bit must be less than end_bit");
                            std::process::exit(1);
                        }
                        analyze(processed_df, AnalysisType::Entropy {
                            start_bit: *start_bit,
                            end_bit: *end_bit,
                        })
                    },
                    frontends::cli::ViewAnalysisCommand::Subnets { max_subnets, prefix_length } => {
                        analyze(processed_df, AnalysisType::Subnets {
                            max_subnets: *max_subnets,
                            prefix_length: *prefix_length,
                        })
                    },
                };

                match result {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                }
                return;
            }
            
            // Default: show interactive TUI
            let df = match field {
                Some(field) => processed_df
                    .lazy()
                    .select([col(field)]),
                None => processed_df
                    .lazy()
            };

            if *tui {
                if let Err(e) = view::run_tui(df) {
                    eprintln!("Error running TUI: {}", e);
                    std::process::exit(1);
                }
            } else {
                print_dataframe(&df.collect().unwrap());
            }
        }
    }
} 