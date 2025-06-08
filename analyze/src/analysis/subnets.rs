use std::collections::HashMap;
use std::net::Ipv6Addr;
use std::fmt;
use crate::{Analysis, PrintableResults};
use polars::prelude::*;
use plugin::contracts::{AbsorbField, MyField};

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
        format!("{:x}/{}", prefix, self.prefix_length)
    }
}

impl AbsorbField<Ipv6Addr> for SubnetAnalysis {
    type Config = SubnetConfig;

    fn absorb(&mut self, config: &Self::Config, item: Ipv6Addr) {
        let prefix_length = config.prefix_length;
        let addr_u128 = u128::from_be_bytes(item.octets());
        let prefix = if prefix_length == 128 {
            addr_u128
        } else {
            addr_u128 >> (128 - prefix_length)
        };
        let subnet = format!("{:x}/{}", prefix, prefix_length);
        *self.subnet_counts.entry(subnet).or_insert(0) += 1;
    }

    fn finalize(&mut self) -> DataFrame {
        let mut subnets: Vec<(String, usize)> = self.subnet_counts.iter().map(|(k, v)| (k.clone(), *v)).collect();
        subnets.sort_by(|a, b| b.1.cmp(&a.1));
        subnets.truncate(self.max_subnets);
        let prefixes: Vec<String> = subnets.iter().map(|(subnet, _)| subnet.clone()).collect();
        let counts: Vec<u64> = subnets.iter().map(|(_, count)| *count as u64).collect();
        DataFrame::new(vec![
            Column::new("subnet".into(), &prefixes),
            Column::new("count".into(), &counts),
        ]).unwrap()
    }
}

#[derive(Debug)]
pub struct SubnetResults {
    pub subnets: Vec<String>,
    pub counts: Vec<u64>,
}

impl SubnetResults {
    pub fn from_dataframe(df: &DataFrame) -> Self {
        let subnets = df.column("subnet").unwrap().str().unwrap().into_no_null_iter().map(|s| s.to_string()).collect();
        let counts = df.column("count").unwrap().u64().unwrap().into_no_null_iter().collect();
        Self { subnets, counts }
    }
}

impl std::fmt::Display for SubnetResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Subnet Analysis Results:")?;
        for (subnet, count) in self.subnets.iter().zip(self.counts.iter()) {
            writeln!(f, "  {}: {}", subnet, count)?;
        }
        Ok(())
    }
}
