use std::net::Ipv6Addr;
use std::fmt;
use hashbrown::HashMap;
use crate::{Analysis, PrintableResults};

#[derive(Debug)]
pub struct StatisticsResults {
    pub total_count: usize,
    pub unique_count: usize,
    pub duplicate_count: usize,
}

impl fmt::Display for StatisticsResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Total addresses: {}", self.total_count)?;
        writeln!(f, "Unique addresses: {}", self.unique_count)?;
        writeln!(f, "Duplicate addresses: {}", self.duplicate_count)?;
        writeln!(f, "Duplication ratio: {:.2}%", 
            if self.total_count > 0 {
                (self.duplicate_count as f64 / self.total_count as f64) * 100.0
            } else {
                0.0
            }
        )
    }
}

impl PrintableResults for StatisticsResults {
    fn print(&self) {
        println!("\nAddress Statistics:");
        println!("{}", self);
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
}

impl Analysis<Ipv6Addr> for StatisticsAnalysis {
    type Results = StatisticsResults;

    #[inline(always)]
    fn absorb(&mut self, addr: Ipv6Addr) {
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

    fn results(self) -> Self::Results {
        let unique_count = self.address_counts.len();
        
        StatisticsResults {
            total_count: self.total_count,
            unique_count,
            duplicate_count: self.total_count - unique_count,
        }
    }
}
