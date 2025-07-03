use futures::stream::{self, Stream, StreamExt};
use rand::Rng;
use std::marker::PhantomData;
use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, Instant};

use probe::Probe;
use pnet::transport::{
    icmp_packet_iter, icmpv6_packet_iter, transport_channel, TransportChannelType, TransportProtocol, TransportReceiver, TransportSender
};

use ipnet::{IpNet, Ipv4Net, Ipv6Net};

pub mod icmp6;
pub mod link_local;

pub struct Scanner2 {
    max_active_probes: usize,
    new_probe_delay: Option<Duration>,
}

impl Scanner2 {
    fn scan<A, T, I>(&self, settings: T, addrs: I)
    where
        A: Copy + Into<IpAddr>,
        T: Probe<A>,
        I: Iterator<Item = A>,
    {
        let mut buffer = [0u8; 1024];
        let mut packet = T::init(&mut buffer);

        let (mut tx, mut rx) = transport_channel(100, T::CHANNEL_TYPE).unwrap();

        for addr in addrs {
            let source = addr.clone();
            let target = addr.clone();
        }
    }
}
