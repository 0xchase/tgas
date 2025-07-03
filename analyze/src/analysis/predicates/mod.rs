pub mod documentation;
pub mod eui64;
pub mod multicast;
pub mod protocols;
pub mod reserved;
pub mod special;
pub mod special_purpose;
pub mod transition;

use plugin::contracts::Predicate;
use std::net::Ipv6Addr;

pub fn get_all_predicates() -> Vec<(&'static str, fn(Ipv6Addr) -> bool)> {
    vec![
        ("loopback", |addr| {
            reserved::LoopbackPredicate.predicate(addr)
        }),
        ("unspecified", |addr| {
            reserved::UnspecifiedPredicate.predicate(addr)
        }),
        ("link_local", |addr| {
            reserved::LinkLocalPredicate.predicate(addr)
        }),
        ("unique_local", |addr| {
            reserved::UniqueLocalPredicate.predicate(addr)
        }),

        ("multicast", |addr| {
            multicast::IsMulticastPredicate.predicate(addr)
        }),
        ("solicited_node", |addr| {
            multicast::SolicitedNodeMulticastPredicate.predicate(addr)
        }),
        ("ipv4_mapped", |addr| {
            transition::Ipv4MappedPredicate.predicate(addr)
        }),
        ("ipv4_to_ipv6", |addr| {
            transition::Ipv4ToIpv6Predicate.predicate(addr)
        }),
        ("extended_ipv4", |addr| {
            transition::ExtendedIpv4Ipv6Predicate.predicate(addr)
        }),
        ("ipv6_to_ipv4", |addr| {
            transition::Ipv6ToIpv4Predicate.predicate(addr)
        }),
        ("documentation", |addr| {
            documentation::DocumentationPredicate.predicate(addr)
        }),
        ("documentation_2", |addr| {
            documentation::Documentation2Predicate.predicate(addr)
        }),
        ("benchmarking", |addr| {
            documentation::BenchmarkingPredicate.predicate(addr)
        }),
        ("teredo", |addr| protocols::TeredoPredicate.predicate(addr)),
        ("ietf_protocol", |addr| {
            protocols::IetfProtocolPredicate.predicate(addr)
        }),
        ("port_control", |addr| {
            protocols::PortControlProtocolPredicate.predicate(addr)
        }),
        ("turn", |addr| protocols::TurnPredicate.predicate(addr)),
        ("dns_sd", |addr| protocols::DnsSdPredicate.predicate(addr)),
        ("amt", |addr| protocols::AmtPredicate.predicate(addr)),
        ("segment_routing", |addr| {
            protocols::SegmentRoutingPredicate.predicate(addr)
        }),
        ("discard_only", |addr| {
            special_purpose::DiscardOnlyPredicate.predicate(addr)
        }),
        ("dummy_prefix", |addr| {
            special_purpose::DummyPrefixPredicate.predicate(addr)
        }),
        ("as112_v6", |addr| {
            special_purpose::As112V6Predicate.predicate(addr)
        }),
        ("direct_as112", |addr| {
            special_purpose::DirectAs112Predicate.predicate(addr)
        }),
        ("deprecated_orchid", |addr| {
            special_purpose::DeprecatedOrchidPredicate.predicate(addr)
        }),
        ("orchid_v2", |addr| {
            special_purpose::OrchidV2Predicate.predicate(addr)
        }),
        ("drone_remote_id", |addr| {
            special_purpose::DroneRemoteIdPredicate.predicate(addr)
        }),
        ("eui64", |addr| eui64::Eui64Analysis.predicate(addr)),
        ("low_byte_host", |addr| {
            eui64::IsLowByteHostPredicate.predicate(addr)
        }),
    ]
}
