use ipnet::Ipv6Net;
use plugin::contracts::{PluginInfo, Predicate};
use std::net::Ipv6Addr;

pub struct DocumentationPredicate;
pub struct Documentation2Predicate;
pub struct BenchmarkingPredicate;

impl PluginInfo for DocumentationPredicate {
    const NAME: &'static str = "documentation_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is documentation (2001:db8::/32)";
}

impl Predicate for DocumentationPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "2001:db8::/32".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for Documentation2Predicate {
    const NAME: &'static str = "documentation_2_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is documentation (3fff::/20)";
}

impl Predicate for Documentation2Predicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "3fff::/20".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for BenchmarkingPredicate {
    const NAME: &'static str = "benchmarking_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is benchmarking (2001:2::/48)";
}

impl Predicate for BenchmarkingPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "2001:2::/48".parse().unwrap();
        network.contains(&addr)
    }
}
