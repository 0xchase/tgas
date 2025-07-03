use plugin::contracts::{AbsorbField, MyField};
use polars::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::net::Ipv6Addr;

#[derive(Default)]
pub struct SubnetConfig {
    pub max_subnets: usize,
    pub prefix_length: u8,
}

pub struct SubnetAnalysis {
    pub subnet_counts: HashMap<String, usize>,
    pub max_subnets: usize,
    pub prefix_length: u8,
}

impl SubnetAnalysis {
    pub fn new_with_options(max_subnets: usize, prefix_length: u8) -> Self {
        Self {
            subnet_counts: HashMap::new(),
            max_subnets,
            prefix_length,
        }
    }
    fn get_subnet(&self, addr: &Ipv6Addr) -> String {
        let addr_u128 = u128::from_be_bytes(addr.octets());
        let prefix = if self.prefix_length == 128 {
            addr_u128
        } else {
            addr_u128 >> (128 - self.prefix_length)
        };
        let subnet_addr = Ipv6Addr::from(prefix << (128 - self.prefix_length));
        format!("{}/{}", subnet_addr, self.prefix_length)
    }
}

impl AbsorbField<Ipv6Addr> for SubnetAnalysis {
    type Config = SubnetConfig;

    fn absorb(&mut self, addr: Ipv6Addr) {
        let subnet = self.get_subnet(&addr);
        *self.subnet_counts.entry(subnet).or_insert(0) += 1;
    }

    fn finalize(&mut self) -> DataFrame {
        let mut subnets: Vec<_> = self.subnet_counts.iter().collect();
        subnets.sort_by(|a, b| b.1.cmp(a.1));
        subnets.truncate(self.max_subnets);

        let subnet_names: Vec<String> = subnets
            .iter()
            .cloned()
            .map(|(name, _)| name.clone())
            .collect();
        let counts: Vec<_> = subnets.iter().map(|(_, count)| **count as u64).collect();

        DataFrame::new(vec![
            Column::new("subnet".into(), &subnet_names),
            Column::new("count".into(), &counts),
        ])
        .unwrap()
    }
}

#[derive(Debug)]
pub struct SubnetResults {
    pub subnets: Vec<(String, usize)>,
}

impl SubnetResults {
    pub fn from_dataframe(df: &polars::prelude::DataFrame) -> Self {
        let subnets: Vec<_> = df
            .column("subnet")
            .unwrap()
            .str()
            .unwrap()
            .into_iter()
            .zip(df.column("count").unwrap().u64().unwrap().into_iter())
            .map(|(name, count)| (name.unwrap().to_string(), count.unwrap() as usize))
            .collect();
        Self { subnets }
    }
}

impl std::fmt::Display for SubnetResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Subnet Analysis Results:")?;
        for (subnet, count) in &self.subnets {
            writeln!(f, "  {}: {}", subnet, count)?;
        }
        Ok(())
    }
}
