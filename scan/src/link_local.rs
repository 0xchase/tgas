use pnet::datalink::{self, NetworkInterface};
use pnet::packet::icmpv6::echo_request::{self, MutableEchoRequestPacket};
use pnet::packet::icmpv6::{self as icmpv6, Icmpv6Types, MutableIcmpv6Packet};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::Packet;
use pnet::transport::{
    self, icmpv6_packet_iter, TransportChannelType, TransportProtocol,
};

use std::net::{IpAddr, Ipv6Addr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Discovers IPv6 hosts on the local network segment using ICMPv6 multicast.
///
/// # Arguments
///
/// * `interface` - The network interface to send the discovery packet on.
///
/// # Returns
///
/// A `Result` containing a `Vec` of discovered `Ipv6Addr`s, or an error string.
pub fn discover_ipv6_link_local(interface: &NetworkInterface) -> Result<Vec<Ipv6Addr>, String> {
    // Find a suitable link-local IPv6 address on the specified interface to use as the source.
    let source_ipv6 = interface
        .ips
        .iter()
        .find(|ip| ip.is_ipv6() && match ip.ip() {
            IpAddr::V6(addr) => addr.octets()[0] == 0xfe && (addr.octets()[1] & 0xc0) == 0x80, // Link-local addresses start with fe80::/10
            _ => false,
        })
        .map(|ip| match ip.ip() {
            IpAddr::V6(addr) => addr,
            _ => unreachable!(),
        })
        .ok_or_else(|| format!("No suitable IPv6 link-local address found on interface {}", interface.name))?;

    // The target is the "all-nodes" link-local multicast address.
    let target_addr = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 1);

    // Create a transport channel. We use a specific transport channel type for ICMPv6.
    let (mut ts, mut tr) = transport::transport_channel(
        4096,
        TransportChannelType::Layer4(TransportProtocol::Ipv6(IpNextHeaderProtocols::Icmpv6)),
    )
    .map_err(|e| format!("Failed to create transport channel: {}", e))?;

    println!("Using source address: {}", source_ipv6);
    println!("Sending discovery packet to multicast address: {}", target_addr);

    // We will store discovered hosts in a thread-safe Set-like structure to avoid duplicates.
    let discovered_hosts = Arc::new(Mutex::new(std::collections::HashSet::new()));
    let discovered_hosts_clone = Arc::clone(&discovered_hosts);

    // --- Receiver Thread ---
    // This thread will listen for incoming ICMPv6 Echo Replies.
    let receiver_thread = thread::spawn(move || {
        let mut iter = icmpv6_packet_iter(&mut tr);
        loop {
            // Wait for up to 1 second for a packet.
            match iter.next_with_timeout(Duration::from_secs(1)) {
                Ok(Some((packet, addr))) => {
                    // We've received a packet, check if it's an Echo Reply.
                    if packet.get_icmpv6_type() == Icmpv6Types::EchoReply {
                        if let Some(echo_reply) = icmpv6::echo_reply::EchoReplyPacket::new(packet.packet()) {
                            // Check if it's a reply to our specific probe.
                            if echo_reply.get_identifier() == 0x1337 {
                                println!("> Received reply from: {}", addr);
                                let mut hosts = discovered_hosts_clone.lock().unwrap();
                                hosts.insert(addr);
                            }
                        }
                    }
                }
                // Timeout means no packet arrived. Continue waiting until the main thread finishes.
                Ok(None) => continue,
                // An error on the channel likely means the program is shutting down.
                Err(_) => break,
            }
        }
    });

    // --- Sender Logic (Main Thread) ---
    // Construct the ICMPv6 Echo Request packet.
    const PAYLOAD_SIZE: usize = 48;
    let mut buffer = [0u8; 8 + PAYLOAD_SIZE]; // 8 byte header + payload
    let mut icmp_packet = MutableEchoRequestPacket::new(&mut buffer).unwrap();

    icmp_packet.set_icmpv6_type(Icmpv6Types::EchoRequest);
    icmp_packet.set_identifier(0x1337);
    icmp_packet.set_sequence_number(0);
    icmp_packet.set_payload(&[0; PAYLOAD_SIZE]);

    // Calculate the checksum. This is required for ICMPv6 and needs the source and destination addresses.
    // Create a separate buffer for checksum calculation
    let mut csum_buffer = [0u8; 8 + PAYLOAD_SIZE];
    csum_buffer.copy_from_slice(&icmp_packet.packet());
    let csum_packet = MutableIcmpv6Packet::new(&mut csum_buffer).unwrap();
    let checksum = icmpv6::checksum(&csum_packet.to_immutable(), &source_ipv6, &target_addr);
    icmp_packet.set_checksum(checksum);

    // Send the single discovery packet to the multicast address.
    if ts.send_to(icmp_packet, IpAddr::V6(target_addr)).is_err() {
        return Err("Failed to send discovery packet".to_string());
    }

    println!("Discovery packet sent. Listening for replies for 5 seconds...");

    // Listen for replies for a fixed duration.
    thread::sleep(Duration::from_secs(5));

    // The listening period is over. We can now collect the results.
    // The receiver thread will exit on its next timeout.
    // We don't strictly need to join it, but it's good practice if we care about its shutdown.
    
    let hosts = discovered_hosts.lock().unwrap();
    let mut results: Vec<Ipv6Addr> = hosts.iter().map(|ip| match ip {
        IpAddr::V6(addr) => *addr,
        _ => unreachable!(),
    }).collect();

    // Sort the results for consistent output.
    results.sort();
    Ok(results)
}

/// Gets a list of usable network interfaces for IPv6 link-local discovery.
///
/// # Returns
///
/// A `Vec` of `NetworkInterface`s that are up, not loopback, and have IPv6 addresses.
pub fn get_usable_interfaces() -> Vec<NetworkInterface> {
    let all_interfaces = datalink::interfaces();
    
    all_interfaces
        .into_iter()
        .filter(|iface| iface.is_up() && !iface.is_loopback() && iface.ips.iter().any(|ip| ip.is_ipv6()))
        .collect()
}

/// Discovers IPv6 hosts on all available interfaces.
///
/// # Returns
///
/// A `Result` containing a `Vec` of discovered `Ipv6Addr`s, or an error string.
pub fn discover_all_ipv6_link_local() -> Result<Vec<Ipv6Addr>, String> {
    let interfaces = get_usable_interfaces();
    
    if interfaces.is_empty() {
        return Err("No active network interfaces with IPv6 found.".to_string());
    }

    let mut all_hosts = std::collections::HashSet::new();
    
    for interface in interfaces {
        println!("Scanning interface: {}", interface.name);
        match discover_ipv6_link_local(&interface) {
            Ok(hosts) => {
                for host in hosts {
                    all_hosts.insert(host);
                }
            }
            Err(e) => {
                println!("Warning: Failed to scan interface {}: {}", interface.name, e);
            }
        }
    }
    
    let mut results: Vec<Ipv6Addr> = all_hosts.into_iter().collect();
    results.sort();
    Ok(results)
} 