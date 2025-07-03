use pnet::datalink::{self, NetworkInterface};
use pnet::packet::Packet;
use pnet::packet::icmpv6::echo_request::{self, MutableEchoRequestPacket};
use pnet::packet::icmpv6::{self as icmpv6, Icmpv6Types, MutableIcmpv6Packet};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::transport::{self, TransportChannelType, TransportProtocol, icmpv6_packet_iter};

use metrics::{counter, gauge};
use std::net::{IpAddr, Ipv6Addr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn discover_ipv6_link_local(interface: &NetworkInterface) -> Result<Vec<Ipv6Addr>, String> {
    let source_ipv6 = interface
        .ips
        .iter()
        .find(|ip| {
            ip.is_ipv6()
                && match ip.ip() {
                    IpAddr::V6(addr) => {
                        addr.octets()[0] == 0xfe && (addr.octets()[1] & 0xc0) == 0x80
                    }
                    _ => false,
                }
        })
        .map(|ip| match ip.ip() {
            IpAddr::V6(addr) => addr,
            _ => unreachable!(),
        })
        .ok_or_else(|| {
            format!(
                "No suitable IPv6 link-local address found on interface {}",
                interface.name
            )
        })?;

    let target_addr = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 1);

    let (mut ts, mut tr) = transport::transport_channel(
        4096,
        TransportChannelType::Layer4(TransportProtocol::Ipv6(IpNextHeaderProtocols::Icmpv6)),
    )
    .map_err(|e| format!("Failed to create transport channel: {}", e))?;

    println!("Using source address: {}", source_ipv6);
    println!(
        "Sending discovery packet to multicast address: {}",
        target_addr
    );

    let discovered_hosts = Arc::new(Mutex::new(std::collections::HashSet::new()));
    let discovered_hosts_clone = Arc::clone(&discovered_hosts);

    let receiver_thread = thread::spawn(move || {
        let mut iter = icmpv6_packet_iter(&mut tr);
        loop {
            match iter.next_with_timeout(Duration::from_secs(1)) {
                Ok(Some((packet, addr))) => {
                    if packet.get_icmpv6_type() == Icmpv6Types::EchoReply {
                        if let Some(echo_reply) =
                            icmpv6::echo_reply::EchoReplyPacket::new(packet.packet())
                        {
                            if echo_reply.get_identifier() == 0x1337 {
                                println!("> Received reply from: {}", addr);
                                let mut hosts = discovered_hosts_clone.lock().unwrap();
                                hosts.insert(addr);
                            }
                        }
                    }
                }
                Ok(None) => continue,
                Err(_) => break,
            }
        }
    });

    const PAYLOAD_SIZE: usize = 48;
    let mut buffer = [0u8; 8 + PAYLOAD_SIZE];
    let mut icmp_packet = MutableEchoRequestPacket::new(&mut buffer).unwrap();

    icmp_packet.set_icmpv6_type(Icmpv6Types::EchoRequest);
    icmp_packet.set_identifier(0x1337);
    icmp_packet.set_sequence_number(0);
    icmp_packet.set_payload(&[0; PAYLOAD_SIZE]);

    let mut csum_buffer = [0u8; 8 + PAYLOAD_SIZE];
    csum_buffer.copy_from_slice(&icmp_packet.packet());
    let csum_packet = MutableIcmpv6Packet::new(&mut csum_buffer).unwrap();
    let checksum = icmpv6::checksum(&csum_packet.to_immutable(), &source_ipv6, &target_addr);
    icmp_packet.set_checksum(checksum);

    if ts.send_to(icmp_packet, IpAddr::V6(target_addr)).is_err() {
        return Err("Failed to send discovery packet".to_string());
    }

    println!("Discovery packet sent. Listening for replies for 5 seconds...");

    thread::sleep(Duration::from_secs(5));



    let hosts = discovered_hosts.lock().unwrap();
    let mut results: Vec<Ipv6Addr> = hosts
        .iter()
        .map(|ip| match ip {
            IpAddr::V6(addr) => *addr,
            _ => unreachable!(),
        })
        .collect();

    results.sort();
    Ok(results)
}

pub fn get_usable_interfaces() -> Vec<NetworkInterface> {
    let all_interfaces = datalink::interfaces();

    all_interfaces
        .into_iter()
        .filter(|iface| {
            iface.is_up() && !iface.is_loopback() && iface.ips.iter().any(|ip| ip.is_ipv6())
        })
        .collect()
}

pub fn discover_all_ipv6_link_local() -> Result<Vec<Ipv6Addr>, String> {
    counter!("rmap_link_local_discoveries_total", 1);
    gauge!("rmap_active_link_local_discoveries", 1.0);

    let interfaces = get_usable_interfaces();

    if interfaces.is_empty() {
        gauge!("rmap_active_link_local_discoveries", 0.0);
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
                println!(
                    "Warning: Failed to scan interface {}: {}",
                    interface.name, e
                );
                counter!("rmap_link_local_interface_errors_total", 1);
            }
        }
    }

    let mut results: Vec<Ipv6Addr> = all_hosts.into_iter().collect();
    results.sort();

    counter!(
        "rmap_link_local_hosts_discovered_total",
        results.len() as u64
    );
    gauge!("rmap_active_link_local_discoveries", 0.0);

    Ok(results)
}
