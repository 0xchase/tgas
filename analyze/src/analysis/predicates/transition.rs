use ipnet::Ipv6Net;
use plugin::contracts::{PluginInfo, Predicate};
use std::net::Ipv6Addr;

pub struct Ipv4MappedPredicate;
pub struct Ipv4ToIpv6Predicate;
pub struct ExtendedIpv4Ipv6Predicate;
pub struct Ipv6ToIpv4Predicate;

impl PluginInfo for Ipv4MappedPredicate {
    const NAME: &'static str = "ipv4_mapped_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is IPv4-mapped (::ffff:0:0/96)";
}

impl Predicate for Ipv4MappedPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "::ffff:0:0/96".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for Ipv4ToIpv6Predicate {
    const NAME: &'static str = "ipv4_to_ipv6_predicate";
    const DESCRIPTION: &'static str =
        "Checks if IPv6 address is IPv4 to IPv6 translation (64:ff9b::/96)";
}

impl Predicate for Ipv4ToIpv6Predicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "64:ff9b::/96".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for ExtendedIpv4Ipv6Predicate {
    const NAME: &'static str = "extended_ipv4_ipv6_predicate";
    const DESCRIPTION: &'static str =
        "Checks if IPv6 address is extended IPv4-IPv6 translation (64:ff9b:1::/48)";
}

impl Predicate for ExtendedIpv4Ipv6Predicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "64:ff9b:1::/48".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for Ipv6ToIpv4Predicate {
    const NAME: &'static str = "ipv6_to_ipv4_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is IPv6 to IPv4 (2002::/16)";
}

impl Predicate for Ipv6ToIpv4Predicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "2002::/16".parse().unwrap();
        network.contains(&addr)
    }
}
