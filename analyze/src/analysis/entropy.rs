use std::collections::HashMap;
use std::net::Ipv6Addr;
use std::fmt;
use crate::{Analysis, PrintableResults};

#[derive(Debug)]
pub struct EntropyResults {
    pub total_count: usize,
    pub unique_count: usize,
    pub total_entropy: f64,
}

impl fmt::Display for EntropyResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Shannon entropy: {:.4} bits", self.total_entropy)
    }
}

impl PrintableResults for EntropyResults {
    fn print(&self) {
        println!("{}", self);
    }
}

pub struct EntropyAnalysis {
    address_counts: HashMap<Ipv6Addr, usize>,
    total_count: usize,
}

impl EntropyAnalysis {
    pub fn new() -> Self {
        Self {
            address_counts: HashMap::new(),
            total_count: 0,
        }
    }

    fn calculate_entropy(&self) -> f64 {
        let mut entropy = 0.0;
        let total = self.total_count as f64;

        for &count in self.address_counts.values() {
            let probability = count as f64 / total;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }
}

impl Analysis<Ipv6Addr> for EntropyAnalysis {
    type Results = EntropyResults;

    fn absorb(&mut self, addr: Ipv6Addr) {
        *self.address_counts.entry(addr).or_insert(0) += 1;
        self.total_count += 1;
    }

    fn results(self) -> Self::Results {
        let unique_count = self.address_counts.len();
        let total_entropy = self.calculate_entropy();

        EntropyResults {
            total_count: self.total_count,
            unique_count,
            total_entropy,
        }
    }
}
