use std::collections::HashMap;
use std::net::Ipv6Addr;
use std::fmt;
use crate::{Analysis, PrintableResults};

const BLUE: &str = "\x1b[34m";
const RESET: &str = "\x1b[0m";

#[derive(Debug)]
pub struct EntropyResults {
    pub total_count: usize,
    pub unique_count: usize,
    pub total_entropy: f64,
    pub start_bit: u8,
    pub end_bit: u8,
}

impl fmt::Display for EntropyResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Shannon entropy (bits {}-{}): {:.4} bits", 
            self.start_bit, self.end_bit, self.total_entropy)?;

        // Only show visualization if we're looking at a specific range
        if self.start_bit > 0 || self.end_bit < 128 {
            // Create a visual representation of the bit range
            let mut addr_str = String::new();
            let mut in_range = false;
            
            for i in 0..32 {
                if i % 4 == 0 {
                    if i > 0 {
                        addr_str.push(':');
                    }
                }
                
                let bit_start = i * 4;
                let bit_end = bit_start + 4;
                let overlaps_range = bit_start < self.end_bit as usize && bit_end > self.start_bit as usize;
                
                if overlaps_range != in_range {
                    addr_str.push_str(if overlaps_range { BLUE } else { RESET });
                    in_range = overlaps_range;
                }
                
                addr_str.push('0');
            }
            
            if in_range {
                addr_str.push_str(RESET);
            }
            
            writeln!(f, "\nBit range visualization:")?;
            writeln!(f, "{}", addr_str)?;
        }
        Ok(())
    }
}

impl PrintableResults for EntropyResults {
    fn print(&self) {
        print!("{}", self);
    }
}

pub struct EntropyAnalysis {
    address_counts: HashMap<u128, usize>,
    total_count: usize,
    start_bit: u8,
    end_bit: u8,
}

impl EntropyAnalysis {
    pub fn new() -> Self {
        Self::new_with_options(0, 128)
    }

    pub fn new_with_options(start_bit: u8, end_bit: u8) -> Self {
        assert!(start_bit < end_bit && end_bit <= 128);
        Self {
            address_counts: HashMap::new(),
            total_count: 0,
            start_bit,
            end_bit,
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

    fn extract_bits(addr: &Ipv6Addr, start_bit: u8, end_bit: u8) -> u128 {
        let addr_u128: u128 = u128::from_be_bytes(addr.octets());
        let width = end_bit - start_bit;
        let mask = if width == 128 {
            u128::MAX
        } else {
            ((1u128 << width) - 1) << (128 - end_bit)
        };
        
        (addr_u128 & mask) >> (128 - end_bit)
    }
}

impl Analysis<Ipv6Addr> for EntropyAnalysis {
    type Results = EntropyResults;

    fn absorb(&mut self, addr: Ipv6Addr) {
        let bits = Self::extract_bits(&addr, self.start_bit, self.end_bit);
        *self.address_counts.entry(bits).or_insert(0) += 1;
        self.total_count += 1;
    }

    fn results(self) -> Self::Results {
        let unique_count = self.address_counts.len();
        let total_entropy = self.calculate_entropy();

        EntropyResults {
            total_count: self.total_count,
            unique_count,
            total_entropy,
            start_bit: self.start_bit,
            end_bit: self.end_bit,
        }
    }
}
