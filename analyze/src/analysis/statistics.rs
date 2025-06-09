use std::net::Ipv6Addr;
use std::fmt;
use hashbrown::HashMap;
use polars::prelude::*;
use plugin::contracts::{AbsorbField, MyField};

#[derive(Default)]
pub struct StatisticsConfig;

pub struct StatisticsAnalysis {
    address_counts: HashMap<Ipv6Addr, usize>,
    total_count: usize,
}

impl StatisticsAnalysis {
    pub fn new() -> Self {
        Self {
            address_counts: HashMap::new(),
            total_count: 0,
        }
    }
}

impl AbsorbField<Ipv6Addr> for StatisticsAnalysis {
    type Config = StatisticsConfig;

    fn absorb(&mut self, addr: Ipv6Addr) {
        *self.address_counts.entry(addr).or_insert(0) += 1;
        self.total_count += 1;
    }

    fn finalize(&mut self) -> DataFrame {
        let unique_count = self.address_counts.len();
        let duplicate_count = self.total_count - unique_count;
        let duplication_ratio = if self.total_count > 0 {
            duplicate_count as f64 / self.total_count as f64
        } else {
            0.0
        };

        DataFrame::new(vec![
            Column::new("total_count".into(), &[self.total_count as u64]),
            Column::new("unique_count".into(), &[unique_count as u64]),
            Column::new("duplicate_count".into(), &[duplicate_count as u64]),
            Column::new("duplication_ratio".into(), &[duplication_ratio]),
        ]).unwrap()
    }
}

#[derive(Debug)]
pub struct StatisticsResults {
    pub total_count: usize,
    pub unique_count: usize,
    pub duplicate_count: usize,
    pub duplication_ratio: f64,
}

impl fmt::Display for StatisticsResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Statistics Analysis Results:")?;
        writeln!(f, "  Total addresses: {}", self.total_count)?;
        writeln!(f, "  Unique addresses: {}", self.unique_count)?;
        writeln!(f, "  Duplicate addresses: {}", self.duplicate_count)?;
        writeln!(f, "  Duplication ratio: {:.2}%", self.duplication_ratio * 100.0)?;
        Ok(())
    }
}

impl StatisticsResults {
    pub fn from_dataframe(df: &polars::prelude::DataFrame) -> Self {
        Self {
            total_count: df.column("total_count").unwrap().u64().unwrap().get(0).unwrap() as usize,
            unique_count: df.column("unique_count").unwrap().u64().unwrap().get(0).unwrap() as usize,
            duplicate_count: df.column("duplicate_count").unwrap().u64().unwrap().get(0).unwrap() as usize,
            duplication_ratio: df.column("duplication_ratio").unwrap().f64().unwrap().get(0).unwrap(),
        }
    }
}
