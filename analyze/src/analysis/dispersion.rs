use std::collections::HashMap;
use std::net::Ipv6Addr;
use std::fmt;
use crate::{Analysis, PrintableResults};

#[derive(Debug)]
pub struct DispersionResults {
    pub total_count: usize,
    pub unique_count: usize,
    pub avg_distance: f64,
    pub max_distance: u128,
    pub coverage_ratio: f64,
}

impl fmt::Display for DispersionResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Total addresses: {}\nUnique addresses: {}\nAverage distance (log2): {:.2}\nMaximum distance: {}\nCoverage ratio: {:.6}", 
            self.total_count, self.unique_count, self.avg_distance, self.max_distance, self.coverage_ratio)
    }
}

impl PrintableResults for DispersionResults {
    fn print(&self) {
        println!("\nDispersion Analysis:");
        println!("{}", self);
    }
}

pub struct DispersionAnalysis {
    address_counts: HashMap<Ipv6Addr, usize>,
    total_count: usize,
}

impl DispersionAnalysis {
    pub fn new() -> Self {
        Self {
            address_counts: HashMap::new(),
            total_count: 0,
        }
    }

    fn calculate_metrics(&self) -> (f64, u128, f64) {
        let mut addr_nums: Vec<u128> = self.address_counts.keys()
            .map(|addr| u128::from_be_bytes(addr.octets()))
            .collect();
        addr_nums.sort_unstable();

        if addr_nums.len() <= 1 {
            return (0.0, 0, 0.0);
        }

        // Calculate average and maximum distance between consecutive addresses
        let mut total_distance = 0u128;
        let mut max_distance = 0u128;
        let mut gaps = Vec::new();

        for window in addr_nums.windows(2) {
            let distance = window[1] - window[0];
            total_distance = total_distance.saturating_add(distance);
            max_distance = max_distance.max(distance);
            gaps.push(distance);
        }

        let avg_distance = if gaps.is_empty() {
            0.0
        } else {
            // Use log scale for the average to handle IPv6's huge address space
            let sum_log_distances: f64 = gaps.iter()
                .map(|&d| (d as f64).log2())
                .sum();
            sum_log_distances / gaps.len() as f64
        };

        // Calculate coverage ratio (how much of the potential address space is used)
        let actual_span = addr_nums.last().unwrap() - addr_nums.first().unwrap();
        let coverage_ratio = if actual_span > 0 {
            (addr_nums.len() as f64) / (actual_span as f64)
        } else {
            0.0
        };

        (avg_distance, max_distance, coverage_ratio)
    }
}

impl Analysis<Ipv6Addr> for DispersionAnalysis {
    type Results = DispersionResults;

    fn absorb(&mut self, addr: Ipv6Addr) {
        *self.address_counts.entry(addr).or_insert(0) += 1;
        self.total_count += 1;
    }

    fn results(self) -> Self::Results {
        let unique_count = self.address_counts.len();
        let (avg_distance, max_distance, coverage_ratio) = self.calculate_metrics();

        DispersionResults {
            total_count: self.total_count,
            unique_count,
            avg_distance,
            max_distance,
            coverage_ratio,
        }
    }
}
