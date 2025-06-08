use std::net::Ipv6Addr;
use std::fmt;
use hashbrown::HashMap;
use polars::prelude::*;
use crate::PrintableResults;
use plugin::contracts::AbsorbField;

#[derive(Debug)]
pub struct StatisticsResults {
    pub total_count: usize,
    pub unique_count: usize,
    pub duplicate_count: usize,
    pub duplication_ratio: f64,
}

impl fmt::Display for StatisticsResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Total addresses: {}", self.total_count)?;
        writeln!(f, "Unique addresses: {}", self.unique_count)?;
        writeln!(f, "Duplicate addresses: {}", self.duplicate_count)?;
        writeln!(f, "Duplication ratio: {:.2}%", self.duplication_ratio * 100.0)
    }
}

impl PrintableResults for StatisticsResults {
    fn print(&self) {
        println!("\nAddress Statistics:");
        println!("{}", self);
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

pub struct StatisticsAnalysis {
    address_counts: HashMap<Ipv6Addr, u32>,
    total_count: usize,
}

impl StatisticsAnalysis {
    pub fn new() -> Self {
        Self {
            address_counts: HashMap::with_capacity(100_000),
            total_count: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            address_counts: HashMap::with_capacity(capacity),
            total_count: 0,
        }
    }

    fn get_results(&self) -> StatisticsResults {
        let unique_count = self.address_counts.len();
        
        StatisticsResults {
            total_count: self.total_count,
            unique_count,
            duplicate_count: self.total_count - unique_count,
            duplication_ratio: if self.total_count > 0 {
                (self.total_count - unique_count) as f64 / self.total_count as f64
            } else {
                0.0
            },
        }
    }
}

impl AbsorbField<Ipv6Addr> for StatisticsAnalysis {
    type Config = ();

    #[inline(always)]
    fn absorb(&mut self, _config: &Self::Config, addr: Ipv6Addr) {
        let entry = self.address_counts.raw_entry_mut().from_key(&addr);
        match entry {
            hashbrown::hash_map::RawEntryMut::Occupied(mut o) => {
                *o.get_mut() += 1;
            }
            hashbrown::hash_map::RawEntryMut::Vacant(v) => {
                v.insert(addr, 1);
            } 
        }
        
        self.total_count += 1;
    }

    fn finalize(&mut self) -> DataFrame {
        let results = self.get_results();
        let total = Column::new("total_count".into(), &[results.total_count as i64]);
        let unique = Column::new("unique_count".into(), &[results.unique_count as i64]);
        let duplicate = Column::new("duplicate_count".into(), &[results.duplicate_count as i64]);
        let ratio = Column::new("duplication_ratio".into(), &[results.duplication_ratio]);

        DataFrame::new(vec![total, unique, duplicate, ratio])
            .expect("Failed to create DataFrame")
    }
}
