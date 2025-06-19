use std::net::Ipv6Addr;
use plugin::contracts::{Predicate, PluginInfo};
use ipnet::Ipv6Net;

pub struct TeredoPredicate;
pub struct IetfProtocolPredicate;
pub struct PortControlProtocolPredicate;
pub struct TurnPredicate;
pub struct DnsSdPredicate;
pub struct AmtPredicate;
pub struct SegmentRoutingPredicate;

// Teredo predicate
impl PluginInfo for TeredoPredicate {
    const NAME: &'static str = "teredo_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is Teredo (2001::/32)";
}

impl Predicate for TeredoPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "2001::/32".parse().unwrap();
        network.contains(&addr)
    }
}

// IETF Protocol predicate
impl PluginInfo for IetfProtocolPredicate {
    const NAME: &'static str = "ietf_protocol_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is IETF protocol (2001::/23)";
}

impl Predicate for IetfProtocolPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "2001::/23".parse().unwrap();
        network.contains(&addr)
    }
}

// Port Control Protocol predicate
impl PluginInfo for PortControlProtocolPredicate {
    const NAME: &'static str = "port_control_protocol_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is Port Control Protocol (2001:1::1/128)";
}

impl Predicate for PortControlProtocolPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "2001:1::1/128".parse().unwrap();
        network.contains(&addr)
    }
}

// TURN predicate
impl PluginInfo for TurnPredicate {
    const NAME: &'static str = "turn_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is TURN (2001:1::2/128)";
}

impl Predicate for TurnPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "2001:1::2/128".parse().unwrap();
        network.contains(&addr)
    }
}

// DNS-SD predicate
impl PluginInfo for DnsSdPredicate {
    const NAME: &'static str = "dns_sd_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is DNS-SD (2001:1::3/128)";
}

impl Predicate for DnsSdPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "2001:1::3/128".parse().unwrap();
        network.contains(&addr)
    }
}

// AMT predicate
impl PluginInfo for AmtPredicate {
    const NAME: &'static str = "amt_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is AMT (2001:3::/32)";
}

impl Predicate for AmtPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "2001:3::/32".parse().unwrap();
        network.contains(&addr)
    }
}

// Segment Routing predicate
impl PluginInfo for SegmentRoutingPredicate {
    const NAME: &'static str = "segment_routing_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is Segment Routing (5f00::/16)";
}

impl Predicate for SegmentRoutingPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "5f00::/16".parse().unwrap();
        network.contains(&addr)
    }
} 