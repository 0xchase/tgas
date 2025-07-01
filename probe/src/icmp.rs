use pnet::packet::icmp::{self, echo_request::MutableEchoRequestPacket, IcmpPacket, IcmpTypes};
use pnet::packet::icmpv6::{self, echo_request::MutableEchoRequestPacket as MutableIcmpv6EchoRequestPacket, Icmpv6Code, Icmpv6Packet, Icmpv6Types};
use pnet::packet::Packet;
use pnet::transport::{self, TransportChannelType, TransportProtocol, TransportSender, TransportReceiver, icmp_packet_iter, icmpv6_packet_iter};
use pnet::packet::ip::IpNextHeaderProtocols;
use std::net::IpAddr;
use std::time::Duration;

use std::net::{Ipv4Addr, Ipv6Addr};
use std::time::Instant;

use crate::Probe;

/// ICMP Echo Request probe implementation (v4 and v6)
#[derive(Debug, Clone)]
pub struct IcmpProbe {
    /// Timeout for the probe in milliseconds
    timeout_ms: u64,
    /// Identifier used in ICMP packets
    identifier: u16,
    /// Payload size for ICMP packets
    payload_size: usize,
}

impl Default for IcmpProbe {
    fn default() -> Self {
        Self {
            timeout_ms: 5000, // 5 second timeout
            identifier: 0x1337,
            payload_size: 48,
        }
    }
}

impl IcmpProbe {
    /// Create a new ICMP probe with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new ICMP probe with custom timeout
    pub fn with_timeout(timeout_ms: u64) -> Self {
        Self {
            timeout_ms,
            ..Default::default()
        }
    }

    /// Create a new ICMP probe with custom settings
    pub fn with_settings(timeout_ms: u64, identifier: u16, payload_size: usize) -> Self {
        Self {
            timeout_ms,
            identifier,
            payload_size,
        }
    }
}

pub struct IcmpPacketIter<'a> {
    inner: pnet::transport::IcmpTransportChannelIterator<'a>,
}

impl<'a> Iterator for IcmpPacketIter<'a> {
    type Item = (IpAddr, Vec<u8>);
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next_with_timeout(Duration::from_secs(2)) {
            Ok(Some((packet, addr))) => Some((addr, packet.packet().to_vec())),
            _ => None,
        }
    }
}

pub struct Icmpv6PacketIter<'a> {
    inner: pnet::transport::Icmpv6TransportChannelIterator<'a>,
}

impl<'a> Iterator for Icmpv6PacketIter<'a> {
    type Item = (IpAddr, Vec<u8>);
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next_with_timeout(Duration::from_secs(2)) {
            Ok(Some((packet, addr))) => Some((addr, packet.packet().to_vec())),
            _ => None,
        }
    }
}

impl Probe<Ipv4Addr> for IcmpProbe {
    const NAME: &'static str = "ICMPv4";
    const DESCRIPTION: &'static str = "ICMPv4 Echo Request probe for IPv4 hosts";
    type PacketType = icmp::echo_request::EchoRequestPacket<'static>;
    type PacketIterator<'a> = IcmpPacketIter<'a>;

    fn build(&self, _source: Ipv4Addr, _target: Ipv4Addr) -> Result<Self::PacketType, String> {
        let buffer_size = MutableEchoRequestPacket::minimum_packet_size() + self.payload_size;
        let mut buffer = vec![0u8; buffer_size];
        let mut icmp_packet = MutableEchoRequestPacket::new(&mut buffer)
            .ok_or("Failed to create mutable ICMP Echo Request packet")?;
        icmp_packet.set_icmp_type(IcmpTypes::EchoRequest);
        icmp_packet.set_identifier(self.identifier);
        icmp_packet.set_sequence_number(0);
        let mut payload = vec![0u8; self.payload_size];
        let now = Instant::now().elapsed().as_millis() as u32;
        if self.payload_size >= 4 {
            payload[..4].copy_from_slice(&now.to_be_bytes());
        }
        icmp_packet.set_payload(&payload);
        let checksum = icmp::checksum(&icmp::IcmpPacket::new(icmp_packet.packet()).unwrap());
        icmp_packet.set_checksum(checksum);
        icmp::echo_request::EchoRequestPacket::owned(buffer).ok_or("Failed to create owned EchoRequestPacket".to_string())
    }

    fn packet_iterator<'a>(&self, receiver: &'a mut TransportReceiver) -> Self::PacketIterator<'a> {
        IcmpPacketIter { inner: icmp_packet_iter(receiver) }
    }
}

impl Probe<Ipv6Addr> for IcmpProbe {
    const NAME: &'static str = "ICMPv6";
    const DESCRIPTION: &'static str = "ICMPv6 Echo Request probe for IPv6 hosts";
    type PacketType = icmpv6::echo_request::EchoRequestPacket<'static>;
    type PacketIterator<'a> = Icmpv6PacketIter<'a>;

    fn build(&self, source: Ipv6Addr, target: Ipv6Addr) -> Result<Self::PacketType, String> {
        let buffer_size = MutableIcmpv6EchoRequestPacket::minimum_packet_size() + self.payload_size;
        let mut buffer = vec![0u8; buffer_size];
        let mut icmpv6_packet = MutableIcmpv6EchoRequestPacket::new(&mut buffer)
            .ok_or("Failed to create mutable ICMPv6 Echo Request packet")?;
        icmpv6_packet.set_identifier(self.identifier);
        icmpv6_packet.set_sequence_number(0);
        let mut payload = vec![0u8; self.payload_size];
        let now = Instant::now().elapsed().as_millis() as u32;
        if self.payload_size >= 4 {
            payload[..4].copy_from_slice(&now.to_be_bytes());
        }
        icmpv6_packet.set_payload(&payload);
        let checksum = icmpv6::checksum(&icmpv6::Icmpv6Packet::new(icmpv6_packet.packet()).unwrap(), &source, &target);
        icmpv6_packet.set_checksum(checksum);
        icmpv6::echo_request::EchoRequestPacket::owned(buffer).ok_or("Failed to create owned Icmpv6EchoRequestPacket".to_string())
    }

    fn packet_iterator<'a>(&self, receiver: &'a mut TransportReceiver) -> Self::PacketIterator<'a> {
        Icmpv6PacketIter { inner: icmpv6_packet_iter(receiver) }
    }
}
