use std::net::Ipv6Addr;
use std::collections::HashMap;
use polars::prelude::*;
use plugin::contracts::AbsorbField;
use ipnet::Ipv6Net;

// Static list of special IPv6 address blocks
const SPECIAL_BLOCKS: &[(&str, &str, &str)] = &[
    ("::1/128", "Loopback Address", "Local loopback address (localhost)"),
    ("::/128", "Unspecified Address", "Unspecified address, used as source when no address is available"),
    ("::ffff:0:0/96", "IPv4-mapped Address", "IPv4 addresses mapped into IPv6 address space"),
    ("64:ff9b::/96", "IPv4-IPv6 Translation", "IPv4/IPv6 translation address block"),
    ("64:ff9b:1::/48", "IPv4-IPv6 Translation", "Extended IPv4/IPv6 translation address block"),
    ("100::/64", "Discard-Only Address Block", "Address block for discarding packets"),
    ("100:0:0:1::/64", "Dummy IPv6 Prefix", "Dummy prefix for testing and documentation"),
    ("2001::/23", "IETF Protocol Assignments", "Address block for IETF protocol assignments"),
    ("2001::/32", "TEREDO", "Teredo tunneling service for IPv6 over IPv4"),
    ("2001:1::1/128", "Port Control Protocol Anycast", "Anycast address for Port Control Protocol"),
    ("2001:1::2/128", "TURN Anycast", "Anycast address for Traversal Using Relays around NAT"),
    ("2001:1::3/128", "DNS-SD Anycast", "Anycast address for DNS Service Discovery"),
    ("2001:2::/48", "Benchmarking", "Address block for network benchmarking"),
    ("2001:3::/32", "AMT", "Automatic Multicast Tunneling service"),
    ("2001:4:112::/48", "AS112-v6", "IPv6 addresses for AS112 DNS service"),
    ("2001:10::/28", "Deprecated ORCHID", "Deprecated Overlay Routable Cryptographic Hash Identifiers"),
    ("2001:20::/28", "ORCHIDv2", "Current version of Overlay Routable Cryptographic Hash Identifiers"),
    ("2001:30::/28", "Drone Remote ID", "Address block for Drone Remote ID Protocol Entity Tags"),
    ("2001:db8::/32", "Documentation", "Address block for documentation and examples"),
    ("2002::/16", "6to4", "6to4 automatic tunneling service"),
    ("2620:4f:8000::/48", "Direct Delegation AS112", "Direct delegation AS112 DNS service"),
    ("3fff::/20", "Documentation", "Address block for documentation and examples"),
    ("5f00::/16", "Segment Routing", "Segment Routing (SRv6) SIDs address block"),
    ("fc00::/7", "Unique-Local", "Unique Local Addresses (ULA) for private networks"),
    ("fe80::/10", "Link-Local", "Link-Local Unicast addresses for local network communication"),
];

#[derive(Debug, Clone)]
pub struct SpecialAddressBlock {
    name: String,
    description: String,
    network: Ipv6Net,
}

#[derive(Debug, Clone)]
pub struct SpecialAnalysis {
    blocks: HashMap<String, SpecialAddressBlock>,
    counts: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct SpecialResults {
    pub block_name: String,
    pub count: usize,
    pub description: String,
}

impl SpecialAnalysis {
    pub fn new() -> Self {
        let mut blocks = HashMap::new();
        
        // Build blocks from the static list
        for (prefix, name, description) in SPECIAL_BLOCKS {
            blocks.insert(prefix.to_string(), SpecialAddressBlock {
                name: name.to_string(),
                description: description.to_string(),
                network: prefix.parse().unwrap(),
            });
        }

        Self {
            blocks,
            counts: HashMap::new(),
        }
    }

    fn matches_block(&self, addr: Ipv6Addr) -> Option<&SpecialAddressBlock> {
        // Check each block's network to see if the address falls within it
        for block in self.blocks.values() {
            if block.network.contains(&addr) {
                return Some(block);
            }
        }
        None
    }
}

impl AbsorbField<Ipv6Addr> for SpecialAnalysis {
    type Config = ();

    fn absorb(&mut self, _config: &Self::Config, addr: Ipv6Addr) {
        if let Some(block) = self.matches_block(addr) {
            *self.counts.entry(block.name.clone()).or_insert(0) += 1;
        }
    }

    fn finalize(&mut self) -> DataFrame {
        let mut block_names = Vec::new();
        let mut counts = Vec::new();
        let mut descriptions = Vec::new();

        // Only include blocks that have at least one match
        for block in self.blocks.values() {
            if let Some(&count) = self.counts.get(&block.name) {
                if count > 0 {
                    block_names.push(block.name.clone());
                    counts.push(count as u64);
                    descriptions.push(block.description.clone());
                }
            }
        }

        DataFrame::new(vec![
            Column::new(PlSmallStr::from("Name"), block_names),
            Column::new(PlSmallStr::from("Count"), counts),
            Column::new(PlSmallStr::from("Description"), descriptions),
        ]).unwrap()
    }
}
