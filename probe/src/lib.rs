use pnet::packet::Packet;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::transport::{
    TransportChannelType, TransportProtocol, TransportReceiver, TransportSender, icmp_packet_iter,
    icmpv6_packet_iter,
};
use std::net::IpAddr;
use std::time::Duration;

mod icmp;
mod tcp;
mod udp;

pub use icmp::IcmpProbe;

#[derive(Debug, Clone)]
pub enum ProbeResult {
    Reachable {
        rtt_ms: u64,
        details: Option<String>,
    },
    Unreachable {
        reason: String,
    },
    Timeout {
        timeout_ms: u64,
    },
    Error {
        error: String,
    },
}

pub trait Probe<T: Clone + Copy + Into<IpAddr>>: Default {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
    const CHANNEL_TYPE: TransportChannelType;

    type Packet<'p>: Packet where Self: 'p;

    fn build2<'a>(&self, buffer: &'a mut [u8], source: T, target: T) {
    }

    fn init<'p>(buffer: &'p mut [u8]) -> Self::Packet<'p>;

    fn update<'p>(&'p self, packet: Self::Packet<'p>, source: T, target: T) -> Result<(), String>;
}
