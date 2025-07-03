use ipnet::Ipv6Net;
use plugin::contracts::{PluginInfo, Predicate};
use std::net::Ipv6Addr;

pub struct DiscardOnlyPredicate;
pub struct DummyPrefixPredicate;
pub struct As112V6Predicate;
pub struct DirectAs112Predicate;
pub struct DeprecatedOrchidPredicate;
pub struct OrchidV2Predicate;
pub struct DroneRemoteIdPredicate;

impl PluginInfo for DiscardOnlyPredicate {
    const NAME: &'static str = "discard_only_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is discard-only (100::/64)";
}

impl Predicate for DiscardOnlyPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "100::/64".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for DummyPrefixPredicate {
    const NAME: &'static str = "dummy_prefix_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is dummy prefix (100:0:0:1::/64)";
}

impl Predicate for DummyPrefixPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "100:0:0:1::/64".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for As112V6Predicate {
    const NAME: &'static str = "as112_v6_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is AS112-v6 (2001:4:112::/48)";
}

impl Predicate for As112V6Predicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "2001:4:112::/48".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for DirectAs112Predicate {
    const NAME: &'static str = "direct_as112_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is Direct AS112 (2620:4f:8000::/48)";
}

impl Predicate for DirectAs112Predicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "2620:4f:8000::/48".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for DeprecatedOrchidPredicate {
    const NAME: &'static str = "deprecated_orchid_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is deprecated ORCHID (2001:10::/28)";
}

impl Predicate for DeprecatedOrchidPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "2001:10::/28".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for OrchidV2Predicate {
    const NAME: &'static str = "orchid_v2_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is ORCHIDv2 (2001:20::/28)";
}

impl Predicate for OrchidV2Predicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "2001:20::/28".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for DroneRemoteIdPredicate {
    const NAME: &'static str = "drone_remote_id_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is Drone Remote ID (2001:30::/28)";
}

impl Predicate for DroneRemoteIdPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "2001:30::/28".parse().unwrap();
        network.contains(&addr)
    }
}
