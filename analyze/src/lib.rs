use std::io::{BufRead, Error as IoError};
use std::net::Ipv6Addr;
use std::collections::HashMap;

#[derive(Debug)]
pub struct AddressStats {
    pub total_count: usize,
    pub unique_count: usize,
    pub duplicate_count: usize,
    pub total_entropy: f64,
    pub avg_distance: f64,
    pub max_distance: u128,
    pub coverage_ratio: f64,
}

pub fn analyze<R: BufRead>(reader: R) -> Result<AddressStats, IoError> {
    let mut address_counts: HashMap<Ipv6Addr, usize> = HashMap::new();
    let mut total_count = 0;

    // Count occurrences of each address
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Ok(addr) = line.parse::<Ipv6Addr>() {
            *address_counts.entry(addr).or_insert(0) += 1;
            total_count += 1;
        }
    }

    // Calculate basic statistics
    let unique_count = address_counts.len();
    let duplicate_count = total_count - unique_count;

    // Calculate entropy across all addresses
    let total_entropy = calculate_address_entropy(&address_counts, total_count);

    // Calculate dispersion metrics
    let (avg_distance, max_distance, coverage_ratio) = if unique_count > 1 {
        calculate_dispersion_metrics(&address_counts)
    } else {
        (0.0, 0, 0.0)
    };

    Ok(AddressStats {
        total_count,
        unique_count,
        duplicate_count,
        total_entropy,
        avg_distance,
        max_distance,
        coverage_ratio,
    })
}

fn calculate_address_entropy(addresses: &HashMap<Ipv6Addr, usize>, total: usize) -> f64 {
    let mut entropy = 0.0;
    let total = total as f64;

    for &count in addresses.values() {
        let probability = count as f64 / total;
        if probability > 0.0 {
            entropy -= probability * probability.log2();
        }
    }

    entropy
}

fn calculate_dispersion_metrics(addresses: &HashMap<Ipv6Addr, usize>) -> (f64, u128, f64) {
    // Convert addresses to u128 for easier arithmetic
    let mut addr_nums: Vec<u128> = addresses.keys()
        .map(|addr| u128::from_be_bytes(addr.octets()))
        .collect();
    addr_nums.sort_unstable();

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
    // We do this by comparing the actual span to the theoretical maximum span
    let actual_span = if addr_nums.len() > 1 {
        addr_nums.last().unwrap() - addr_nums.first().unwrap()
    } else {
        0
    };

    // Coverage ratio is the number of addresses divided by the span size
    // This gives us a measure of how densely the addresses fill their range
    let coverage_ratio = if actual_span > 0 {
        (addr_nums.len() as f64) / (actual_span as f64)
    } else {
        0.0
    };

    (avg_distance, max_distance, coverage_ratio)
}
