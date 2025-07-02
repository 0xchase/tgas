// This file is deprecated. Predicates have been reorganized into the following modules:
// - multicast.rs: IsMulticastPredicate, SolicitedNodeMulticastPredicate
// - reserved.rs: LoopbackPredicate, UnspecifiedPredicate, LinkLocalPredicate, UniqueLocalPredicate
// - transition.rs: Ipv4MappedPredicate, Ipv4ToIpv6Predicate, ExtendedIpv4Ipv6Predicate, Ipv6ToIpv4Predicate
// - documentation.rs: DocumentationPredicate, Documentation2Predicate, BenchmarkingPredicate
// - protocols.rs: TeredoPredicate, IetfProtocolPredicate, PortControlProtocolPredicate, TurnPredicate, DnsSdPredicate, AmtPredicate, SegmentRoutingPredicate
// - special_purpose.rs: DiscardOnlyPredicate, DummyPrefixPredicate, As112V6Predicate, DirectAs112Predicate, DeprecatedOrchidPredicate, OrchidV2Predicate, DroneRemoteIdPredicate

// Re-export all predicates for backward compatibility
pub use crate::analysis::predicates::documentation::*;
pub use crate::analysis::predicates::multicast::*;
pub use crate::analysis::predicates::protocols::*;
pub use crate::analysis::predicates::reserved::*;
pub use crate::analysis::predicates::special_purpose::*;
pub use crate::analysis::predicates::transition::*;
