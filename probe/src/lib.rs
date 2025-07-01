use std::net::IpAddr;
use pnet::transport::{TransportSender, TransportReceiver, icmp_packet_iter, icmpv6_packet_iter};
use pnet::packet::Packet;
use std::time::Duration;

mod icmp;
mod tcp;
mod udp;

pub use icmp::IcmpProbe;

#[derive(Debug, Clone)]
pub enum ProbeResult {
    /// Target is reachable and responded
    Reachable {
        /// Round trip time in milliseconds
        rtt_ms: u64,
        /// Additional probe-specific data
        details: Option<String>,
    },
    /// Target is unreachable (no response)
    Unreachable {
        /// Reason for unreachability
        reason: String,
    },
    /// Probe timed out
    Timeout {
        /// Timeout duration in milliseconds
        timeout_ms: u64,
    },
    /// Error occurred during probing
    Error {
        /// Error description
        error: String,
    },
}

pub trait Probe<T: Clone + Copy + Into<IpAddr>> {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
    type PacketType: pnet::packet::Packet;
    type PacketIterator<'a>: Iterator<Item = (IpAddr, Vec<u8>)> + 'a where Self: 'a;

    fn build(&self, source: T, target: T) -> Result<Self::PacketType, String>;
    
    /// Send the built packet to the target using the provided transport sender
    fn send(&self, source: &T, target: &T, sender: &mut TransportSender) -> Result<(), String> {
        let packet = self.build(source.clone(), target.clone())?;
        sender.send_to(packet, (*target).into())
            .map_err(|e| format!("Failed to send packet: {}", e))?;
        Ok(())
    }

    /// Create a packet iterator for this probe type
    fn packet_iterator<'a>(&self, receiver: &'a mut TransportReceiver) -> Self::PacketIterator<'a>;

    /// Receive the next packet from the transport receiver
    fn recv(&self, receiver: &mut TransportReceiver) -> Result<Option<(IpAddr, Vec<u8>)>, String> {
        let mut iter = self.packet_iterator(receiver);
        match iter.next() {
            Some((addr, packet_data)) => Ok(Some((addr, packet_data))),
            None => Ok(None),
        }
    }
}