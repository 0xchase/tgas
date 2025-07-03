use pnet::packet::Packet;
use pnet::packet::icmp::{self, IcmpPacket, IcmpTypes, echo_request::MutableEchoRequestPacket};
use pnet::packet::icmpv6::{
    self, Icmpv6Code, Icmpv6Packet, Icmpv6Types,
    echo_request::MutableEchoRequestPacket as MutableIcmpv6EchoRequestPacket,
};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::transport::{TransportChannelType, TransportProtocol};
use std::net::IpAddr;
use std::time::Duration;

use std::net::{Ipv4Addr, Ipv6Addr};
use std::time::Instant;

use crate::Probe;

#[derive(Debug, Clone)]
pub struct IcmpProbe {
    timeout_ms: u64,
    identifier: u16,
    payload_size: usize,
}

impl Default for IcmpProbe {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            identifier: 0x1337,
            payload_size: 48,
        }
    }
}

impl IcmpProbe {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timeout(timeout_ms: u64) -> Self {
        Self {
            timeout_ms,
            ..Default::default()
        }
    }

    pub fn with_settings(timeout_ms: u64, identifier: u16, payload_size: usize) -> Self {
        Self {
            timeout_ms,
            identifier,
            payload_size,
        }
    }
}

impl Probe<Ipv4Addr> for IcmpProbe {
    const NAME: &'static str = "ICMPv4";
    const DESCRIPTION: &'static str = "ICMPv4 Echo Request probe for IPv4 hosts";
    
    const CHANNEL_TYPE: TransportChannelType =
        TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Icmp));

    type Packet<'p> = MutableEchoRequestPacket<'p>;

    fn init<'p>(buffer: &'p mut [u8]) -> Self::Packet<'p> {
        Self::Packet::new(buffer).unwrap()
    }

    fn update<'p>(&'p self, mut packet: Self::Packet<'p>, _source: Ipv4Addr, _target: Ipv4Addr) -> Result<(), String> {
        packet.set_icmp_type(IcmpTypes::EchoRequest);
        packet.set_identifier(self.identifier);
        packet.set_sequence_number(0);

        let payload: [u8; 5] = [0; 5];
        packet.set_payload(&payload);

        let data = packet.packet();
        let icmp = icmp::IcmpPacket::new(data).unwrap();
        let checksum = icmp::checksum(&icmp);
        packet.set_checksum(checksum);

        Ok(())
    }
}

impl Probe<Ipv6Addr> for IcmpProbe {
    const NAME: &'static str = "ICMPv6";
    const DESCRIPTION: &'static str = "ICMPv6 Echo Request probe for IPv6 hosts";
    const CHANNEL_TYPE: TransportChannelType =
        TransportChannelType::Layer4(TransportProtocol::Ipv6(IpNextHeaderProtocols::Icmpv6));

    type Packet<'p> = MutableIcmpv6EchoRequestPacket<'p>;

    fn init<'p>(buffer: &'p mut [u8]) -> Self::Packet<'p> {
        Self::Packet::new(buffer).unwrap()
    }

    fn update<'p>(&'p self, mut packet: Self::Packet<'p>, source: Ipv6Addr, target: Ipv6Addr) -> Result<(), String> {
        packet.set_identifier(self.identifier);
        packet.set_sequence_number(0);

        let payload: [u8; 5] = [0; 5];
        packet.set_payload(&payload);

        let data = packet.packet();
        let icmp = icmpv6::Icmpv6Packet::new(data).unwrap();
        let checksum = icmpv6::checksum(&icmp, &source, &target);
        packet.set_checksum(checksum);

        Ok(())
    }
}
