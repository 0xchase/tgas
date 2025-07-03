use itertools::Itertools;
use plugin::contracts::{AbsorbField, MyField};
use polars::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::net::Ipv6Addr;

#[derive(Debug)]
pub struct DispersionResults {
    pub min_distance: u32,
    pub max_distance: u32,
    pub avg_distance: f64,
    pub total_pairs: u64,
}

impl DispersionResults {
    pub fn from_dataframe(df: &polars::prelude::DataFrame) -> Self {
        Self {
            min_distance: df
                .column("min_distance")
                .unwrap()
                .u32()
                .unwrap()
                .get(0)
                .unwrap(),
            max_distance: df
                .column("max_distance")
                .unwrap()
                .u32()
                .unwrap()
                .get(0)
                .unwrap(),
            avg_distance: df
                .column("avg_distance")
                .unwrap()
                .f64()
                .unwrap()
                .get(0)
                .unwrap(),
            total_pairs: df
                .column("total_pairs")
                .unwrap()
                .u64()
                .unwrap()
                .get(0)
                .unwrap(),
        }
    }
}

impl std::fmt::Display for DispersionResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Dispersion Analysis Results:")?;
        writeln!(f, "  Minimum distance: {}", self.min_distance)?;
        writeln!(f, "  Maximum distance: {}", self.max_distance)?;
        writeln!(f, "  Average distance: {:.2}", self.avg_distance)?;
        writeln!(f, "  Total pairs: {}", self.total_pairs)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct DispersionConfig;

pub struct DispersionAnalysis {
    addresses: Vec<Ipv6Addr>,
}

impl DispersionAnalysis {
    pub fn new() -> Self {
        Self {
            addresses: Vec::new(),
        }
    }
}

impl AbsorbField<Ipv6Addr> for DispersionAnalysis {
    type Config = DispersionConfig;

    fn absorb(&mut self, addr: Ipv6Addr) {
        self.addresses.push(addr);
    }

    fn finalize(&mut self) -> DataFrame {
        let mut min_distance = u32::MAX;
        let mut max_distance = 0u32;
        let mut total_distance = 0u64;
        let mut pair_count = 0u64;

        for (a, b) in self.addresses.iter().combinations(2).map(|v| (v[0], v[1])) {
            let a_u128 = u128::from_be_bytes(a.octets());
            let b_u128 = u128::from_be_bytes(b.octets());
            let dist = (a_u128 ^ b_u128).count_ones();
            min_distance = min_distance.min(dist);
            max_distance = max_distance.max(dist);
            total_distance = total_distance.wrapping_add(dist as u64);
            pair_count += 1;
        }

        let avg_distance = if pair_count > 0 {
            total_distance as f64 / pair_count as f64
        } else {
            0.0
        };

        self.addresses.clear();

        DataFrame::new(vec![
            Column::new("min_distance".into(), &[min_distance]),
            Column::new("max_distance".into(), &[max_distance]),
            Column::new("avg_distance".into(), &[avg_distance]),
            Column::new("total_pairs".into(), &[pair_count]),
        ])
        .unwrap()
    }
}
