use futures::stream::{self, Stream, StreamExt};
use rand::Rng;
use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, Instant};
use tokio::time::sleep;

use probe::Probe;
use pnet::transport::{
    icmp_packet_iter, icmpv6_packet_iter, transport_channel, TransportChannelType, TransportProtocol, TransportReceiver, TransportSender
};

use ipnet::{IpNet, Ipv4Net, Ipv6Net};

pub mod icmp6;
pub mod link_local;

pub async fn test_scan() {
    let addrs = vec![
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 3)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 4)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 5)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 6)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 7)),
    ];

    let addrs = (0..10)
        .map(|i| IpAddr::V4(Ipv4Addr::new(192, 168, 1, i)))
        .collect::<Vec<IpAddr>>();

    let net4: IpNet = "10.1.1.0/24".parse().unwrap();
    // let net6: Ipv6Net = "fd00::/24".parse().unwrap();
    // println!("net4: {:?}", net4.hosts().count());
    // println!("net6: {:?}", net6.hosts().nth(123487234));

    let start = Instant::now();
    let scanner = Scanner::default();
    let mut results = scanner.scan(net4.hosts().take(10));

    println!("Starting scan");
    while let Some(result) = results.next().await {
        println!("Recieved {} after {:?}", result.addr, start.elapsed());
    }

    println!("Scan complete");
}

pub struct Scanner2<A: Into<IpAddr>, T: Probe<A>> {
    max_active_probes: usize,
    new_probe_delay: Option<Duration>,
}

impl<A: Into<IpAddr>, T: Probe<A>> Scanner2<A, T> {
    fn scan<I>(&self, settings: T, addrs: I)
    where
        I: Iterator<Item = A>,
    {
        let (mut tx, mut rx) = transport_channel(100, T::CHANNEL_TYPE).unwrap();
        for addr in addrs {
            let packet = T::build(&settings, addr, addr).unwrap();
            tx.send_to(packet, addr.into());
        }
    }
}

/*impl<T: Probe<Ipv6Addr>> Scanner2<T> {
    fn scan<I>(&self, addrs: I) -> impl Stream<Item = ProbeResult>
    where
        I: Iterator<Item = Ipv6Addr>,
    {
        let initial_state = addrs.peekable();
    }
}

impl<T: Probe<IpAddr>> Scanner2<T> {
    fn scan<I>(&self, addrs: I) -> impl Stream<Item = ProbeResult>
    where
        I: Iterator<Item = IpAddr>,
    {
        let probe = T::default();
        for addr in addrs {
            
        }
        let initial_state = addrs.peekable();
    }
}*/

pub struct Scanner {
    max_active_probes: usize,
    new_probe_delay: Option<Duration>,
}

impl Default for Scanner {
    fn default() -> Self {
        Self {
            max_active_probes: usize::MAX,
            new_probe_delay: None,
        }
    }
}

impl Scanner {
    fn scan<I>(&self, addrs_iter: I) -> impl Stream<Item = ProbeResult>
    where
        I: Iterator<Item = IpAddr>,
    {
        // The iterator is now part of the state for unfold
        let initial_state = addrs_iter.peekable();

        let stream = stream::unfold(
            initial_state, // Pass the peekable iterator as the initial state
            // The closure takes the current state (the iterator)
            move |mut iter_state| {
                // This outer async block is the future that unfold polls
                async move {
                    // Peek to see if there's an item without consuming it from iter_state yet
                    if iter_state.peek().is_some() {
                        // If an item exists, now consume it
                        let addr = iter_state.next().unwrap();
                        let probe_future = IcmpProbe::execute_probe(addr);

                        // If there are more addresses to schedule after this one, sleep.
                        // iter_state.peek() checks the *next* item.
                        if let Some(delay) = self.new_probe_delay {
                            if iter_state.peek().is_some() {
                                sleep(delay).await;
                            }
                        }

                        // Yield the probe future and the (potentially advanced) iterator state
                        Some((probe_future, iter_state))
                    } else {
                        // No more addresses, end the stream by returning None
                        None
                    }
                }
            },
        )
        .buffer_unordered(self.max_active_probes);

        Box::pin(stream) // Pin the resulting stream to make it Unpin
    }
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub addr: IpAddr,
}

struct IcmpProbe {}

impl IcmpProbe {
    async fn execute_probe(addr: IpAddr) -> ProbeResult {
        let probe_start_time = Instant::now();
        let operational_delay = rand::thread_rng().gen_range(100..800);
        println!("Probing {}", addr);
        sleep(Duration::from_millis(operational_delay)).await;
        ProbeResult { addr }
    }
}
