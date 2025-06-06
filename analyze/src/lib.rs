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

#[derive(Debug)]
pub struct CsvAddressStats {
    pub total_count: usize,
    pub unique_count: usize,
    pub duplicate_count: usize,
    pub total_entropy: f64,
    pub avg_distance: f64,
    pub max_distance: u128,
    pub coverage_ratio: f64,
    pub active_response_ratio: f64,
}

#[derive(Debug)]
pub enum AnalyzeResult {
    IpList(AddressStats),
    Csv(CsvAddressStats),
}

pub fn analyze<R: BufRead>(mut reader: R) -> Result<AnalyzeResult, IoError> {
    // Peek at the first non-empty, non-comment line to determine file type
    let mut first_line = String::new();
    while reader.read_line(&mut first_line)? > 0 {
        let trimmed = first_line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            break;
        }
        first_line.clear();
    }
    let is_csv = first_line.contains(",");
    // Rewind the reader by re-creating it with the first line included
    let mut lines = vec![first_line];
    for line in reader.lines() {
        lines.push(line?);
    }
    let cursor = std::io::Cursor::new(lines.join("\n"));
    let buf_reader = std::io::BufReader::new(cursor);
    if is_csv {
        analyze_csv(buf_reader).map(AnalyzeResult::Csv)
    } else {
        analyze_ip_list(buf_reader).map(AnalyzeResult::IpList)
    }
}

fn analyze_csv<R: BufRead>(mut reader: R) -> Result<CsvAddressStats, IoError> {
    use std::io::BufRead;
    use std::net::Ipv6Addr;
    use std::collections::HashMap;
    let mut header = String::new();
    // Read header
    let n = reader.read_line(&mut header)?;
    if n == 0 {
        return Err(IoError::from(std::io::ErrorKind::UnexpectedEof));
    }
    let header = header.trim();
    let columns: Vec<&str> = header.split(',').collect();
    let saddr_idx = columns.iter().position(|&c| c == "saddr");
    let type_idx = columns.iter().position(|&c| c == "type");
    if saddr_idx.is_none() {
        return Err(IoError::new(std::io::ErrorKind::InvalidData, "No saddr column in CSV header"));
    }
    let saddr_idx = saddr_idx.unwrap();
    let mut address_counts: HashMap<Ipv6Addr, usize> = HashMap::new();
    let mut total_count = 0;
    let mut active_count = 0;
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() <= saddr_idx {
            continue;
        }
        if let Ok(addr) = fields[saddr_idx].parse::<Ipv6Addr>() {
            *address_counts.entry(addr).or_insert(0) += 1;
            total_count += 1;
            // Check for active response if type column exists
            if let Some(type_idx) = type_idx {
                if fields.len() > type_idx && fields[type_idx] == "129" {
                    active_count += 1;
                }
            }
        }
    }
    let unique_count = address_counts.len();
    let duplicate_count = total_count - unique_count;
    let total_entropy = calculate_address_entropy(&address_counts, total_count);
    let (avg_distance, max_distance, coverage_ratio) = if unique_count > 1 {
        calculate_dispersion_metrics(&address_counts)
    } else {
        (0.0, 0, 0.0)
    };
    let active_response_ratio = if total_count > 0 { active_count as f64 / total_count as f64 } else { 0.0 };
    Ok(CsvAddressStats {
        total_count,
        unique_count,
        duplicate_count,
        total_entropy,
        avg_distance,
        max_distance,
        coverage_ratio,
        active_response_ratio,
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

fn analyze_ip_list<R: BufRead>(reader: R) -> Result<AddressStats, IoError> {
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
    let unique_count = address_counts.len();
    let duplicate_count = total_count - unique_count;
    let total_entropy = calculate_address_entropy(&address_counts, total_count);
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
