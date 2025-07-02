use plugin::contracts::{AbsorbField, MyField};
use polars::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::net::Ipv6Addr;

pub struct ShannonEntropyConfig {
    pub start_bit: u8,
    pub end_bit: u8,
}

pub struct ShannonEntropyAnalysis {
    start_bit: u8,
    end_bit: u8,
    bit_counts: HashMap<u8, usize>,
    total_bits: usize,
}

impl ShannonEntropyAnalysis {
    pub fn new_with_options(start_bit: u8, end_bit: u8) -> Self {
        Self {
            start_bit,
            end_bit,
            bit_counts: HashMap::new(),
            total_bits: 0,
        }
    }
}

impl AbsorbField<Ipv6Addr> for ShannonEntropyAnalysis {
    type Config = ShannonEntropyConfig;

    fn absorb(&mut self, addr: Ipv6Addr) {
        let bytes = addr.octets();
        for i in self.start_bit..self.end_bit {
            let byte_idx = (i / 8) as usize;
            let bit_idx = i % 8;
            if byte_idx < bytes.len() {
                let bit = (bytes[byte_idx] >> bit_idx) & 1;
                *self.bit_counts.entry(bit).or_insert(0) += 1;
                self.total_bits += 1;
            }
        }
    }

    fn finalize(&mut self) -> DataFrame {
        let mut entropy = 0.0;
        for count in self.bit_counts.values() {
            let p = *count as f64 / self.total_bits as f64;
            entropy -= p * p.log2();
        }

        DataFrame::new(vec![
            Column::new("entropy".into(), &[entropy]),
            Column::new("total_bits".into(), &[self.total_bits as u64]),
            Column::new(
                "bit_distribution".into(),
                &[format!("{:?}", self.bit_counts)],
            ),
        ])
        .unwrap()
    }
}

#[derive(Debug)]
pub struct ShannonEntropyResults {
    pub entropy: f64,
    pub total_bits: usize,
    pub bit_distribution: String,
}

impl ShannonEntropyResults {
    pub fn from_dataframe(df: &polars::prelude::DataFrame) -> Self {
        Self {
            entropy: df.column("entropy").unwrap().f64().unwrap().get(0).unwrap(),
            total_bits: df
                .column("total_bits")
                .unwrap()
                .u64()
                .unwrap()
                .get(0)
                .unwrap() as usize,
            bit_distribution: df
                .column("bit_distribution")
                .unwrap()
                .str()
                .unwrap()
                .get(0)
                .unwrap()
                .to_string(),
        }
    }
}

impl fmt::Display for ShannonEntropyResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Shannon Entropy Analysis Results:")?;
        writeln!(f, "  Entropy: {:.4} bits", self.entropy)?;
        writeln!(f, "  Total bits analyzed: {}", self.total_bits)?;
        writeln!(f, "  Bit distribution: {}", self.bit_distribution)?;
        Ok(())
    }
}
