use ipnet::Ipv6Net;
use plugin::contracts::{PluginInfo, Predicate};
use std::net::Ipv6Addr;

pub struct IsMulticastPredicate;
pub struct SolicitedNodeMulticastPredicate;

impl PluginInfo for IsMulticastPredicate {
    const NAME: &'static str = "is_multicast_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is multicast (ff00::/8)";
}

impl Predicate for IsMulticastPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "ff00::/8".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for SolicitedNodeMulticastPredicate {
    const NAME: &'static str = "solicited_node_multicast_predicate";
    const DESCRIPTION: &'static str =
        "Checks if IPv6 address is a solicited-node multicast address (ff02::1:ff00:0000/104)";
}

impl Predicate for SolicitedNodeMulticastPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "ff02::1:ff00:0000/104".parse().unwrap();
        network.contains(&addr)
    }
}
