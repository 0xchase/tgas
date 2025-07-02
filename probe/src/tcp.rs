// tcp_syn, tcp_ack

use crate::Probe;
use pnet::packet::Packet;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::tcp::{self, MutableTcpPacket, TcpFlags, TcpPacket};
use pnet::transport::{
    self, TransportChannelType, TransportProtocol, TransportReceiver, TransportSender,
    tcp_packet_iter,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Instant;

/// TCP SYN probe implementation
#[derive(Debug, Clone)]
pub struct TcpSynProbe {
    /// Timeout for the probe in milliseconds
    timeout_ms: u64,
    /// Source port (0 for random)
    source_port: u16,
    /// Target port to probe
    target_port: u16,
    /// TCP window size
    window_size: u16,
}

impl Default for TcpSynProbe {
    fn default() -> Self {
        Self {
            timeout_ms: 5000, // 5 second timeout
            source_port: 0,   // Random source port
            target_port: 80,  // Default to HTTP port
            window_size: 1024,
        }
    }
}

impl TcpSynProbe {
    /// Create a new TCP SYN probe with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new TCP SYN probe with custom target port
    pub fn with_port(target_port: u16) -> Self {
        Self {
            target_port,
            ..Default::default()
        }
    }

    /// Create a new TCP SYN probe with custom settings
    pub fn with_settings(
        timeout_ms: u64,
        source_port: u16,
        target_port: u16,
        window_size: u16,
    ) -> Self {
        Self {
            timeout_ms,
            source_port,
            target_port,
            window_size,
        }
    }
}

/// TCP ACK probe implementation
#[derive(Debug, Clone)]
pub struct TcpAckProbe {
    /// Timeout for the probe in milliseconds
    timeout_ms: u64,
    /// Source port (0 for random)
    source_port: u16,
    /// Target port to probe
    target_port: u16,
    /// TCP window size
    window_size: u16,
}

impl Default for TcpAckProbe {
    fn default() -> Self {
        Self {
            timeout_ms: 5000, // 5 second timeout
            source_port: 0,   // Random source port
            target_port: 80,  // Default to HTTP port
            window_size: 1024,
        }
    }
}

impl TcpAckProbe {
    /// Create a new TCP ACK probe with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new TCP ACK probe with custom target port
    pub fn with_port(target_port: u16) -> Self {
        Self {
            target_port,
            ..Default::default()
        }
    }

    /// Create a new TCP ACK probe with custom settings
    pub fn with_settings(
        timeout_ms: u64,
        source_port: u16,
        target_port: u16,
        window_size: u16,
    ) -> Self {
        Self {
            timeout_ms,
            source_port,
            target_port,
            window_size,
        }
    }
}

pub struct TcpPacketIter<'a> {
    inner: pnet::transport::TcpTransportChannelIterator<'a>,
}

impl<'a> Iterator for TcpPacketIter<'a> {
    type Item = (IpAddr, Vec<u8>);
    fn next(&mut self) -> Option<Self::Item> {
        match self
            .inner
            .next_with_timeout(std::time::Duration::from_secs(2))
        {
            Ok(Some((packet, addr))) => Some((addr, packet.packet().to_vec())),
            _ => None,
        }
    }
}

impl Probe<Ipv4Addr> for TcpSynProbe {
    const NAME: &'static str = "TCP_SYN_v4";
    const DESCRIPTION: &'static str = "TCP SYN probe for IPv4 hosts";
    const CHANNEL_TYPE: TransportChannelType =
        TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp));
    type PacketIterator<'a> = TcpPacketIter<'a>;

    fn build(&self, _source: Ipv4Addr, _target: Ipv4Addr) -> Result<impl Packet, String> {
        let buffer_size = MutableTcpPacket::minimum_packet_size();
        let mut buffer = vec![0u8; buffer_size];
        let mut tcp_packet =
            MutableTcpPacket::new(&mut buffer).ok_or("Failed to create mutable TCP packet")?;

        // Set source port (random if 0)
        let source_port = if self.source_port == 0 {
            (Instant::now().elapsed().as_nanos() % 65536) as u16
        } else {
            self.source_port
        };
        tcp_packet.set_source(source_port);
        tcp_packet.set_destination(self.target_port);

        // Set sequence number (random)
        let seq = (Instant::now().elapsed().as_nanos() % u64::MAX as u128) as u32;
        tcp_packet.set_sequence(seq);

        // Set acknowledgment number to 0 for SYN
        tcp_packet.set_acknowledgement(0);

        // Set flags: SYN
        tcp_packet.set_flags(TcpFlags::SYN);

        // Set window size
        tcp_packet.set_window(self.window_size);

        // Set urgent pointer to 0
        tcp_packet.set_urgent_ptr(0);

        // Calculate checksum (will be calculated by the transport layer)
        tcp_packet.set_checksum(0);

        tcp::TcpPacket::owned(buffer).ok_or("Failed to create owned TcpPacket".to_string())
    }

    fn packet_iterator<'a>(&self, receiver: &'a mut TransportReceiver) -> Self::PacketIterator<'a> {
        TcpPacketIter {
            inner: tcp_packet_iter(receiver),
        }
    }
}

impl Probe<Ipv6Addr> for TcpSynProbe {
    const NAME: &'static str = "TCP_SYN_v6";
    const DESCRIPTION: &'static str = "TCP SYN probe for IPv6 hosts";
    const CHANNEL_TYPE: TransportChannelType =
        TransportChannelType::Layer4(TransportProtocol::Ipv6(IpNextHeaderProtocols::Tcp));
    type PacketIterator<'a> = TcpPacketIter<'a>;

    fn build(&self, _source: Ipv6Addr, _target: Ipv6Addr) -> Result<impl Packet, String> {
        let buffer_size = MutableTcpPacket::minimum_packet_size();
        let mut buffer = vec![0u8; buffer_size];
        let mut tcp_packet =
            MutableTcpPacket::new(&mut buffer).ok_or("Failed to create mutable TCP packet")?;

        // Set source port (random if 0)
        let source_port = if self.source_port == 0 {
            (Instant::now().elapsed().as_nanos() % 65536) as u16
        } else {
            self.source_port
        };
        tcp_packet.set_source(source_port);
        tcp_packet.set_destination(self.target_port);

        // Set sequence number (random)
        let seq = (Instant::now().elapsed().as_nanos() % u64::MAX as u128) as u32;
        tcp_packet.set_sequence(seq);

        // Set acknowledgment number to 0 for SYN
        tcp_packet.set_acknowledgement(0);

        // Set flags: SYN
        tcp_packet.set_flags(TcpFlags::SYN);

        // Set window size
        tcp_packet.set_window(self.window_size);

        // Set urgent pointer to 0
        tcp_packet.set_urgent_ptr(0);

        // Calculate checksum (will be calculated by the transport layer)
        tcp_packet.set_checksum(0);

        tcp::TcpPacket::owned(buffer).ok_or("Failed to create owned TcpPacket".to_string())
    }

    fn packet_iterator<'a>(&self, receiver: &'a mut TransportReceiver) -> Self::PacketIterator<'a> {
        TcpPacketIter {
            inner: tcp_packet_iter(receiver),
        }
    }
}

impl Probe<Ipv4Addr> for TcpAckProbe {
    const NAME: &'static str = "TCP_ACK_v4";
    const DESCRIPTION: &'static str = "TCP ACK probe for IPv4 hosts";
    const CHANNEL_TYPE: TransportChannelType =
        TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp));
    type PacketIterator<'a> = TcpPacketIter<'a>;

    fn build(&self, _source: Ipv4Addr, _target: Ipv4Addr) -> Result<impl Packet, String> {
        let buffer_size = MutableTcpPacket::minimum_packet_size();
        let mut buffer = vec![0u8; buffer_size];
        let mut tcp_packet =
            MutableTcpPacket::new(&mut buffer).ok_or("Failed to create mutable TCP packet")?;

        // Set source port (random if 0)
        let source_port = if self.source_port == 0 {
            (Instant::now().elapsed().as_nanos() % 65536) as u16
        } else {
            self.source_port
        };
        tcp_packet.set_source(source_port);
        tcp_packet.set_destination(self.target_port);

        // Set sequence number (random)
        let seq = (Instant::now().elapsed().as_nanos() % u64::MAX as u128) as u32;
        tcp_packet.set_sequence(seq);

        // Set acknowledgment number (random for ACK probe)
        let ack = (Instant::now().elapsed().as_nanos() % u64::MAX as u128) as u32;
        tcp_packet.set_acknowledgement(ack);

        // Set flags: ACK
        tcp_packet.set_flags(TcpFlags::ACK);

        // Set window size
        tcp_packet.set_window(self.window_size);

        // Set urgent pointer to 0
        tcp_packet.set_urgent_ptr(0);

        // Calculate checksum (will be calculated by the transport layer)
        tcp_packet.set_checksum(0);

        tcp::TcpPacket::owned(buffer).ok_or("Failed to create owned TcpPacket".to_string())
    }

    fn packet_iterator<'a>(&self, receiver: &'a mut TransportReceiver) -> Self::PacketIterator<'a> {
        TcpPacketIter {
            inner: tcp_packet_iter(receiver),
        }
    }
}

impl Probe<Ipv6Addr> for TcpAckProbe {
    const NAME: &'static str = "TCP_ACK_v6";
    const DESCRIPTION: &'static str = "TCP ACK probe for IPv6 hosts";
    const CHANNEL_TYPE: TransportChannelType =
        TransportChannelType::Layer4(TransportProtocol::Ipv6(IpNextHeaderProtocols::Tcp));
    type PacketIterator<'a> = TcpPacketIter<'a>;

    fn build(&self, _source: Ipv6Addr, _target: Ipv6Addr) -> Result<impl Packet, String> {
        let buffer_size = MutableTcpPacket::minimum_packet_size();
        let mut buffer = vec![0u8; buffer_size];
        let mut tcp_packet =
            MutableTcpPacket::new(&mut buffer).ok_or("Failed to create mutable TCP packet")?;

        // Set source port (random if 0)
        let source_port = if self.source_port == 0 {
            (Instant::now().elapsed().as_nanos() % 65536) as u16
        } else {
            self.source_port
        };
        tcp_packet.set_source(source_port);
        tcp_packet.set_destination(self.target_port);

        // Set sequence number (random)
        let seq = (Instant::now().elapsed().as_nanos() % u64::MAX as u128) as u32;
        tcp_packet.set_sequence(seq);

        // Set acknowledgment number (random for ACK probe)
        let ack = (Instant::now().elapsed().as_nanos() % u64::MAX as u128) as u32;
        tcp_packet.set_acknowledgement(ack);

        // Set flags: ACK
        tcp_packet.set_flags(TcpFlags::ACK);

        // Set window size
        tcp_packet.set_window(self.window_size);

        // Set urgent pointer to 0
        tcp_packet.set_urgent_ptr(0);

        // Calculate checksum (will be calculated by the transport layer)
        tcp_packet.set_checksum(0);

        tcp::TcpPacket::owned(buffer).ok_or("Failed to create owned TcpPacket".to_string())
    }

    fn packet_iterator<'a>(&self, receiver: &'a mut TransportReceiver) -> Self::PacketIterator<'a> {
        TcpPacketIter {
            inner: tcp_packet_iter(receiver),
        }
    }
}
