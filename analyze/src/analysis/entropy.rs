use std::collections::HashMap;
use std::net::Ipv6Addr;
use std::fmt;
use crate::{Analysis, PrintableResults};
use polars::prelude::*;
use plugin::contracts::{AbsorbField, MyField};

const BLUE: &str = "\x1b[34m";
const RESET: &str = "\x1b[0m";

#[derive(Default)]
pub struct EntropyConfig {
    pub start_bit: u8,
    pub end_bit: u8,
}

pub struct EntropyAnalysis {
    pub entropy: f64,
    pub count: usize,
    pub start_bit: u8,
    pub end_bit: u8,
}

impl EntropyAnalysis {
    pub fn new_with_options(start_bit: u8, end_bit: u8) -> Self {
        Self {
            entropy: 0.0,
            count: 0,
            start_bit,
            end_bit,
        }
    }
}

impl AbsorbField<Ipv6Addr> for EntropyAnalysis {
    type Config = EntropyConfig;

    fn absorb(&mut self, config: &Self::Config, item: Ipv6Addr) {
        let bytes = item.octets();
        // For simplicity, just sum the bits in the selected range
        let mut bit_sum = 0u32;
        for bit in config.start_bit..config.end_bit {
            let byte_idx = (bit / 8) as usize;
            let bit_idx = bit % 8;
            bit_sum += ((bytes[byte_idx] >> (7 - bit_idx)) & 1) as u32;
        }
        self.entropy += bit_sum as f64 / (config.end_bit - config.start_bit) as f64;
        self.count += 1;
    }

    fn finalize(&mut self) -> DataFrame {
        let avg_entropy = if self.count > 0 {
            self.entropy / self.count as f64
        } else {
            0.0
        };
        DataFrame::new(vec![
            Column::new("entropy".into(), &[avg_entropy]),
            Column::new("count".into(), &[self.count as u64]),
        ]).unwrap()
    }
}

#[derive(Debug)]
pub struct EntropyResults {
    pub entropy: f64,
    pub count: u64,
}

impl EntropyResults {
    pub fn from_dataframe(df: &DataFrame) -> Self {
        Self {
            entropy: df.column("entropy").unwrap().f64().unwrap().get(0).unwrap(),
            count: df.column("count").unwrap().u64().unwrap().get(0).unwrap(),
        }
    }
}

impl fmt::Display for EntropyResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Entropy Analysis Results:")?;
        writeln!(f, "  Entropy: {:.4}", self.entropy)?;
        writeln!(f, "  Count: {}", self.count)?;
        Ok(())
    }
}

impl PrintableResults for EntropyResults {
    fn print(&self) {
        print!("{}", self);
    }
}
