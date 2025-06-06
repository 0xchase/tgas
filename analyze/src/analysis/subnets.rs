use std::collections::HashMap;
use std::net::Ipv6Addr;
use std::fmt;
use crate::{Analysis, PrintableResults};

#[derive(Debug)]
pub struct SubnetResults {
    pub total_count: usize,
    pub unique_count: usize,
    pub subnet_counts: HashMap<String, usize>,
    pub max_subnets: usize,
    pub prefix_length: u8,
}

impl fmt::Display for SubnetResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Total addresses: {}", self.total_count)?;
        writeln!(f, "Unique addresses: {}", self.unique_count)?;
        writeln!(f, "Number of /{} subnets: {}", self.prefix_length, self.subnet_counts.len())?;
        
        let mut subnets: Vec<_> = self.subnet_counts.iter().collect();
        subnets.sort_by(|a, b| b.1.cmp(a.1));
        
        writeln!(f, "\nTop {} /{} subnets:", self.max_subnets, self.prefix_length)?;
        for (i, (subnet, count)) in subnets.iter().take(self.max_subnets).enumerate() {
            writeln!(f, "{}. {}: {} addresses", i + 1, subnet, count)?;
        }
        Ok(())
    }
}

impl PrintableResults for SubnetResults {
    fn print(&self) {
        println!("\nSubnet Analysis:");
        println!("{}", self);
    }
}

pub struct SubnetAnalysis {
    address_counts: HashMap<Ipv6Addr, usize>,
    subnet_counts: HashMap<String, usize>,
    total_count: usize,
    max_subnets: usize,
    prefix_length: u8,
}

impl SubnetAnalysis {
    pub fn new() -> Self {
        Self::new_with_options(10, 64)
    }

    pub fn new_with_options(max_subnets: usize, prefix_length: u8) -> Self {
        Self {
            address_counts: HashMap::new(),
            subnet_counts: HashMap::new(),
            total_count: 0,
            max_subnets,
            prefix_length,
        }
    }

    fn get_subnet(&self, addr: &Ipv6Addr) -> String {
        let addr_u128 = u128::from(*addr);
        let prefix_bits = self.prefix_length as u32;
        let shift = 128 - prefix_bits;
        let prefix_mask = if shift == 128 { 0 } else { !0u128 >> shift << shift };
        let prefix = addr_u128 & prefix_mask;
        
        // Format the prefix in IPv6 format
        let octets = prefix.to_be_bytes();
        let mut segments = [0u16; 8];
        for i in 0..8 {
            segments[i] = u16::from_be_bytes([octets[i*2], octets[i*2+1]]);
        }
        
        // Find the last non-zero segment for compact display
        let mut last_nonzero = 7;
        while last_nonzero > 0 && segments[last_nonzero] == 0 {
            last_nonzero -= 1;
        }
        
        // Format the prefix string
        let mut result = String::new();
        for i in 0..=last_nonzero {
            if i > 0 {
                result.push(':');
            }
            result.push_str(&format!("{:x}", segments[i]));
        }
        result.push_str(&format!("::/{}",self.prefix_length));
        result
    }
}

impl Analysis<Ipv6Addr> for SubnetAnalysis {
    type Results = SubnetResults;

    fn absorb(&mut self, addr: Ipv6Addr) {
        *self.address_counts.entry(addr).or_insert(0) += 1;
        self.total_count += 1;

        let subnet = self.get_subnet(&addr);
        *self.subnet_counts.entry(subnet).or_insert(0) += 1;
    }

    fn results(self) -> Self::Results {
        let unique_count = self.address_counts.len();

        SubnetResults {
            total_count: self.total_count,
            unique_count,
            subnet_counts: self.subnet_counts,
            max_subnets: self.max_subnets,
            prefix_length: self.prefix_length,
        }
    }
}
