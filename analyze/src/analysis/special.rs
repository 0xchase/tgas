use std::net::Ipv6Addr;
use std::collections::HashMap;
use polars::prelude::*;
use plugin::contracts::AbsorbField;
use ipnet::Ipv6Net;

// Static list of special IPv6 address blocks
const SPECIAL_BLOCKS: &[(&str, &str)] = &[
    ("::1/128", "Loopback"),
    ("::/128", "Unspecified"),
    ("::ffff:0:0/96", "IPv4-Mapped"),
    ("64:ff9b::/96", "IPv4 to IPv6"),
    ("64:ff9b:1::/48", "Extended IPv4-IPv6 Translation"),
    ("100::/64", "Discard-Only"),
    ("100:0:0:1::/64", "Dummy Prefix"),
    ("2001::/23", "IETF Protocol"),
    ("2001::/32", "Teredo"),
    ("2001:1::1/128", "Port Control Protocol"),
    ("2001:1::2/128", "TURN"),
    ("2001:1::3/128", "DNS-SD"),
    ("2001:2::/48", "Benchmarking"),
    ("2001:3::/32", "AMT"),
    ("2001:4:112::/48", "AS112-v6"),
    ("2001:10::/28", "Deprecated ORCHID"),
    ("2001:20::/28", "ORCHIDv2"),
    ("2001:30::/28", "Drone Remote ID"),
    ("2002::/16", "IPv6 to IPv4"),
    ("2620:4f:8000::/48", "Direct AS112"),
    ("2001:db8::/32", "Documentation"),
    ("3fff::/20", "Documentation"),
    ("5f00::/16", "Segment Routing"),
    ("fc00::/7", "Unique Local"),
    ("fe80::/10", "Link Local"),
];

#[derive(Debug, Clone)]
pub struct SpecialAddressBlock {
    name: String,
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
        for (prefix, name) in SPECIAL_BLOCKS {
            blocks.insert(prefix.to_string(), SpecialAddressBlock {
                name: name.to_string(),
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

    fn absorb(&mut self, addr: Ipv6Addr) {
        if let Some(block) = self.matches_block(addr) {
            *self.counts.entry(block.name.clone()).or_insert(0) += 1;
        }
    }

    fn finalize(&mut self) -> DataFrame {
        let mut prefixes = Vec::new();
        let mut block_names = Vec::new();
        let mut counts = Vec::new();

        // Only include blocks that have at least one match
        for block in self.blocks.values() {
            if let Some(&count) = self.counts.get(&block.name) {
                if count > 0 {
                    prefixes.push(block.network.to_string());
                    block_names.push(block.name.clone());
                    counts.push(count as u64);
                }
            }
        }

        let sort_options = SortMultipleOptions::default().with_order_descending(true);

        DataFrame::new(vec![
            Column::new(PlSmallStr::from("Prefix"), prefixes),
            Column::new(PlSmallStr::from("Name"), block_names),
            Column::new(PlSmallStr::from("Count"), counts),
        ]).unwrap().sort(vec!["Count"], sort_options).unwrap()
    }
}
