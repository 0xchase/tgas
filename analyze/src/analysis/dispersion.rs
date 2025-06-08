use std::collections::HashMap;
use std::net::Ipv6Addr;
use std::fmt;
use crate::{Analysis, PrintableResults};
use polars::prelude::*;
use plugin::contracts::{AbsorbField, MyField};
use itertools::Itertools;

#[derive(Debug)]
pub struct DispersionResults {
    pub min_distance: u64,
    pub max_distance: u64,
    pub avg_distance: f64,
    pub total_pairs: u64,
}

impl fmt::Display for DispersionResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Total addresses: {}\nUnique addresses: {}\nAverage distance (log2): {:.2}\nMaximum distance: {}\nCoverage ratio: {:.6}", 
            self.total_pairs, self.total_pairs, self.avg_distance, self.max_distance, self.avg_distance)
    }
}

impl PrintableResults for DispersionResults {
    fn print(&self) {
        println!("\nDispersion Analysis:");
        println!("{}", self);
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

    fn absorb(&mut self, _config: &Self::Config, addr: Ipv6Addr) {
        self.addresses.push(addr);
    }

    fn finalize(&mut self) -> DataFrame {
        // Calculate dispersion metrics
        let mut distances = Vec::new();
        for i in 0..self.addresses.len() {
            for j in (i + 1)..self.addresses.len() {
                let dist = self.addresses[i].segments()
                    .iter()
                    .zip(self.addresses[j].segments().iter())
                    .map(|(a, b)| (a ^ b).count_ones() as u64)
                    .sum::<u64>();
                distances.push(dist);
            }
        }

        // Create DataFrame with dispersion metrics
        let min_dist = distances.iter().min().unwrap_or(&0);
        let max_dist = distances.iter().max().unwrap_or(&0);
        let avg_dist = distances.iter().sum::<u64>() as f64 / distances.len() as f64;

        DataFrame::new(vec![
            Column::new("min_distance".into(), &[*min_dist]),
            Column::new("max_distance".into(), &[*max_dist]),
            Column::new("avg_distance".into(), &[avg_dist]),
            Column::new("total_pairs".into(), &[distances.len() as u64]),
        ]).unwrap()
    }
}

impl Analysis<Ipv6Addr> for DispersionAnalysis {
    type Results = DispersionResults;

    fn absorb(&mut self, addr: Ipv6Addr) {
        self.addresses.push(addr);
    }

    fn results(self) -> Self::Results {
        let min_distance = self.addresses.iter().min().unwrap().segments()
            .iter()
            .zip(self.addresses.iter().max().unwrap().segments().iter())
            .map(|(a, b)| (a ^ b).count_ones() as u64)
            .sum();
        let max_distance = self.addresses.iter().max().unwrap().segments()
            .iter()
            .zip(self.addresses.iter().min().unwrap().segments().iter())
            .map(|(a, b)| (a ^ b).count_ones() as u64)
            .sum();
        let avg_distance = self.addresses.iter()
            .flat_map(|addr| addr.segments().iter())
            .combinations(2)
            .map(|(a, b)| (a ^ b).count_ones() as f64)
            .sum::<f64>() / self.addresses.len() as f64;
        let total_pairs = self.addresses.len() as u64 * (self.addresses.len() - 1) as u64 / 2;

        DispersionResults {
            min_distance,
            max_distance,
            avg_distance,
            total_pairs,
        }
    }
}

impl DispersionResults {
    pub fn from_dataframe(df: &DataFrame) -> Self {
        Self {
            min_distance: df.column("min_distance").unwrap().u64().unwrap().get(0).unwrap(),
            max_distance: df.column("max_distance").unwrap().u64().unwrap().get(0).unwrap(),
            avg_distance: df.column("avg_distance").unwrap().f64().unwrap().get(0).unwrap(),
            total_pairs: df.column("total_pairs").unwrap().u64().unwrap().get(0).unwrap(),
        }
    }
}

impl std::fmt::Display for DispersionResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Dispersion Analysis Results:")?;
        writeln!(f, "  Minimum Distance: {}", self.min_distance)?;
        writeln!(f, "  Maximum Distance: {}", self.max_distance)?;
        writeln!(f, "  Average Distance: {:.2}", self.avg_distance)?;
        writeln!(f, "  Total Address Pairs: {}", self.total_pairs)?;
        Ok(())
    }
}
