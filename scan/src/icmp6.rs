use pnet::packet::Packet;
use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::icmp::{self, IcmpTypes, MutableIcmpPacket};
use pnet::packet::icmpv6::echo_request::MutableEchoRequestPacket as MutableIcmpv6EchoRequestPacket;
use pnet::packet::icmpv6::{self, Icmpv6Types, MutableIcmpv6Packet};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::transport::{
    self, TransportChannelType, TransportProtocol, TransportReceiver, TransportSender,
    icmp_packet_iter, icmpv6_packet_iter,
};

use metrics::{counter, gauge, histogram};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

/// Represents the result of a single probe.
#[derive(Debug)]
pub struct ProbeResult {
    pub addr: IpAddr,
    pub rtt: Duration,
}

pub fn icmp4_scan(network: ipnet::Ipv4Net) -> Vec<ProbeResult> {
    println!("Starting ICMPv4 scan of network: {}", network);

    // Record scan start
    counter!("ipv6kit_icmp4_scans_total", 1);
    gauge!("ipv6kit_active_icmp4_scans", 1.0);

    let (mut ts, mut tr) = transport::transport_channel(
        4096,
        TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Icmp)),
    )
    .expect("Failed to create transport channel");

    let (tx, rx) = std::sync::mpsc::channel();

    let receiver_thread = std::thread::spawn(move || {
        icmp4_receiver_thread(&mut tr, tx);
    });

    let source_ip = network.addr();
    let hosts: Vec<Ipv4Addr> = network.hosts().collect();
    let host_count = hosts.len();
    println!("Sending {} ICMPv4 Echo Requests...", host_count);

    // Record total hosts to scan
    counter!("ipv6kit_icmp4_hosts_total", host_count as u64);

    for (i, host) in hosts.into_iter().enumerate() {
        send_icmpv4_echo_request(&mut ts, source_ip, host);
        std::thread::sleep(Duration::from_millis(20));

        if (i + 1) % 50 == 0 {
            println!("Sent {}/{} requests", i + 1, host_count);
        }
    }

    println!("All packets sent. Waiting for remaining responses...");
    drop(ts);

    receiver_thread.join().unwrap();

    let results: Vec<ProbeResult> = rx.try_iter().collect();

    // Record scan results
    counter!("ipv6kit_icmp4_responses_total", results.len() as u64);
    if host_count > 0 {
        let response_rate = results.len() as f64 / host_count as f64;
        gauge!("ipv6kit_icmp4_response_rate", response_rate);
    }
    gauge!("ipv6kit_active_icmp4_scans", 0.0);

    println!(
        "ICMPv4 scan complete. Found {} responsive hosts.",
        results.len()
    );
    results
}

fn icmp4_receiver_thread(tr: &mut TransportReceiver, tx: Sender<ProbeResult>) {
    let mut iter = icmp_packet_iter(tr);
    loop {
        match iter.next_with_timeout(Duration::from_secs(2)) {
            Ok(Some((packet, addr))) => {
                if packet.get_icmp_type() == IcmpTypes::EchoReply {
                    if let Some(echo_reply) =
                        icmp::echo_reply::EchoReplyPacket::new(packet.packet())
                    {
                        if echo_reply.get_identifier() == 0x1337 {
                            // Check payload length for our timestamp
                            if !echo_reply.payload().is_empty() {
                                let sent_time = u32::from_be_bytes(
                                    echo_reply.payload()[0..4].try_into().unwrap(),
                                );
                                let now = Instant::now().elapsed().as_millis() as u32;
                                let rtt_ms = now.saturating_sub(sent_time);
                                let rtt = Duration::from_millis(rtt_ms as u64);

                                println!(
                                    "Received ICMPv4 Echo Reply from {} (RTT: {:?})",
                                    addr, rtt
                                );

                                let result = ProbeResult { addr, rtt };
                                if tx.send(result).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            Ok(None) => {
                println!("Receiver timed out. Scan complete.");
                break;
            }
            Err(_) => {
                println!("Receiver channel closed. Exiting.");
                break;
            }
        }
    }
}

fn send_icmpv4_echo_request(sender: &mut TransportSender, _source_ip: Ipv4Addr, dest_ip: Ipv4Addr) {
    // ICMP Header (8 bytes) + Payload (48 bytes)
    const PAYLOAD_SIZE: usize = 48;
    let mut buffer = [0u8; 8 + PAYLOAD_SIZE];
    let mut icmp_packet = MutableEchoRequestPacket::new(&mut buffer).unwrap();

    icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
    icmp_packet.set_identifier(0x1337);
    icmp_packet.set_sequence_number(0);

    // Create the large payload, embedding our timestamp
    let mut payload = [0u8; PAYLOAD_SIZE];
    let now = Instant::now().elapsed().as_millis() as u32;
    payload[0..4].copy_from_slice(&now.to_be_bytes());
    icmp_packet.set_payload(&payload);

    // *** THIS IS THE CRITICAL FIX ***
    // Manually calculate the checksum and set it on the packet.
    // Create a separate buffer for checksum calculation
    let mut csum_buffer = [0u8; 8 + PAYLOAD_SIZE];
    csum_buffer.copy_from_slice(&icmp_packet.packet());
    let csum_packet = icmp::MutableIcmpPacket::new(&mut csum_buffer).unwrap();
    let checksum = icmp::checksum(&csum_packet.to_immutable());
    icmp_packet.set_checksum(checksum);

    if sender.send_to(icmp_packet, dest_ip.into()).is_err() {
        eprintln!("Error sending ICMPv4 packet to {}", dest_ip);
    }
}

pub fn icmp6_scan(network: ipnet::Ipv6Net) -> Vec<ProbeResult> {
    println!("Starting ICMPv6 scan of network: {}", network);

    // Record scan start
    counter!("ipv6kit_icmp6_scans_total", 1);
    gauge!("ipv6kit_active_icmp6_scans", 1.0);

    let (mut ts, mut tr) = transport::transport_channel(
        4096,
        TransportChannelType::Layer4(TransportProtocol::Ipv6(IpNextHeaderProtocols::Icmpv6)),
    )
    .expect("Failed to create transport channel");

    let (tx, rx) = std::sync::mpsc::channel();

    let receiver_thread = std::thread::spawn(move || {
        icmpv6_receiver_thread(&mut tr, tx);
    });

    let source_ip = network.addr();
    let hosts: Vec<Ipv6Addr> = network.hosts().collect();
    let host_count = hosts.len();
    println!("Sending {} ICMPv6 Echo Requests...", host_count);

    // Record total hosts to scan
    counter!("ipv6kit_icmp6_hosts_total", host_count as u64);

    for (i, host) in hosts.into_iter().enumerate() {
        send_icmpv6_echo_request(&mut ts, source_ip, host);
        std::thread::sleep(Duration::from_millis(20));

        if (i + 1) % 50 == 0 {
            println!("Sent {}/{} requests", i + 1, host_count);
        }
    }

    println!("All packets sent. Waiting for remaining responses...");
    drop(ts);

    receiver_thread.join().unwrap();

    let results: Vec<ProbeResult> = rx.try_iter().collect();

    // Record scan results
    counter!("ipv6kit_icmp6_responses_total", results.len() as u64);
    if host_count > 0 {
        let response_rate = results.len() as f64 / host_count as f64;
        gauge!("ipv6kit_icmp6_response_rate", response_rate);
    }
    gauge!("ipv6kit_active_icmp6_scans", 0.0);

    println!(
        "ICMPv6 scan complete. Found {} responsive hosts.",
        results.len()
    );
    results
}

fn icmpv6_receiver_thread(tr: &mut TransportReceiver, tx: Sender<ProbeResult>) {
    let mut iter = icmpv6_packet_iter(tr);
    loop {
        match iter.next_with_timeout(Duration::from_secs(2)) {
            Ok(Some((packet, addr))) => {
                if let Some(echo_reply) = icmpv6::echo_reply::EchoReplyPacket::new(packet.packet())
                {
                    if echo_reply.get_identifier() == 0x1337 {
                        // Check payload length for our timestamp
                        if !echo_reply.payload().is_empty() {
                            let sent_time =
                                u32::from_be_bytes(echo_reply.payload()[0..4].try_into().unwrap());
                            let now = Instant::now().elapsed().as_millis() as u32;
                            let rtt_ms = now.saturating_sub(sent_time);
                            let rtt = Duration::from_millis(rtt_ms as u64);

                            println!("Received ICMPv6 Echo Reply from {} (RTT: {:?})", addr, rtt);

                            let result = ProbeResult {
                                addr: addr.into(),
                                rtt,
                            };
                            if tx.send(result).is_err() {
                                break;
                            }
                        }
                    }
                }
            }
            Ok(None) => {
                println!("Receiver timed out. Scan complete.");
                break;
            }
            Err(_) => {
                println!("Receiver channel closed. Exiting.");
                break;
            }
        }
    }
}

fn send_icmpv6_echo_request(sender: &mut TransportSender, source_ip: Ipv6Addr, dest_ip: Ipv6Addr) {
    // ICMPv6 Header (8 bytes) + Payload (48 bytes)
    const PAYLOAD_SIZE: usize = 48;
    let mut buffer = [0u8; 8 + PAYLOAD_SIZE];
    let mut icmp_packet = MutableIcmpv6EchoRequestPacket::new(&mut buffer).unwrap();

    icmp_packet.set_identifier(0x1337);
    icmp_packet.set_sequence_number(0);

    // Create the large payload, embedding our timestamp
    let mut payload = [0u8; PAYLOAD_SIZE];
    let now = Instant::now().elapsed().as_millis() as u32;
    payload[0..4].copy_from_slice(&now.to_be_bytes());
    icmp_packet.set_payload(&payload);

    let mut csum_buffer = [0u8; 8 + PAYLOAD_SIZE];
    csum_buffer.copy_from_slice(&icmp_packet.packet());
    let csum_packet = icmpv6::MutableIcmpv6Packet::new(&mut csum_buffer).unwrap();
    let checksum = icmpv6::checksum(&csum_packet.to_immutable(), &source_ip, &dest_ip);
    icmp_packet.set_checksum(checksum);

    if sender.send_to(icmp_packet, dest_ip.into()).is_err() {
        eprintln!("Error sending ICMPv6 packet to {}", dest_ip);
    }
}
