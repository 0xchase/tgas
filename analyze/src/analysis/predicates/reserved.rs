use ipnet::Ipv6Net;
use plugin::contracts::{PluginInfo, Predicate};
use std::net::Ipv6Addr;

pub struct LoopbackPredicate;
pub struct UnspecifiedPredicate;
pub struct LinkLocalPredicate;
pub struct UniqueLocalPredicate;
pub struct IsGloballyRoutablePredicate;

impl PluginInfo for LoopbackPredicate {
    const NAME: &'static str = "loopback_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is loopback (::1/128)";
}

impl Predicate for LoopbackPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "::1/128".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for UnspecifiedPredicate {
    const NAME: &'static str = "unspecified_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is unspecified (::/128)";
}

impl Predicate for UnspecifiedPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "::/128".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for LinkLocalPredicate {
    const NAME: &'static str = "link_local_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is Link Local (fe80::/10)";
}

impl Predicate for LinkLocalPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "fe80::/10".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for UniqueLocalPredicate {
    const NAME: &'static str = "unique_local_predicate";
    const DESCRIPTION: &'static str = "Checks if IPv6 address is Unique Local (fc00::/7)";
}

impl Predicate for UniqueLocalPredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let network: Ipv6Net = "fc00::/7".parse().unwrap();
        network.contains(&addr)
    }
}

impl PluginInfo for IsGloballyRoutablePredicate {
    const NAME: &'static str = "is_globally_routable_predicate";
    const DESCRIPTION: &'static str = "Checks if the address is globally routable (i.e., not private, loopback, link-local, documentation, etc.).";
}

impl Predicate for IsGloballyRoutablePredicate {
    type In = Ipv6Addr;

    fn predicate(&self, addr: Self::In) -> bool {
        let loopback_pred = LoopbackPredicate;
        let unspecified_pred = UnspecifiedPredicate;
        let link_local_pred = LinkLocalPredicate;
        let unique_local_pred = UniqueLocalPredicate;

        use crate::analysis::predicates::documentation::{
            Documentation2Predicate, DocumentationPredicate,
        };
        use crate::analysis::predicates::multicast::IsMulticastPredicate;
        use crate::analysis::predicates::transition::Ipv4MappedPredicate;

        let multicast_pred = IsMulticastPredicate;
        let ipv4_mapped_pred = Ipv4MappedPredicate;
        let documentation_pred = DocumentationPredicate;
        let documentation2_pred = Documentation2Predicate;

        !loopback_pred.predicate(addr)
            && !unspecified_pred.predicate(addr)
            && !link_local_pred.predicate(addr)
            && !unique_local_pred.predicate(addr)
            && !multicast_pred.predicate(addr)
            && !ipv4_mapped_pred.predicate(addr)
            && !documentation_pred.predicate(addr)
            && !documentation2_pred.predicate(addr)
    }
}
