use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv6Addr;
use std::str::FromStr;
use polars::prelude::*;

pub fn open_csv_lazy(file: &PathBuf, field: &Option<String>) -> Result<LazyFrame, String> {
    LazyCsvReader::new(file)
        .with_infer_schema_length(Some(100))
        .with_has_header(true)
        .with_chunk_size(10000)
        .finish()
        .map_err(|e| format!("Failed to parse CSV file: {}", e))
        .map(|lf| match field {
            Some(field) => lf.select([col(field)]),
            None => lf
        }.with_new_streaming(true))
}

pub fn load_file(file: &PathBuf, field: &Option<String>) -> DataFrame {
    let mut lf = open_csv_lazy(file, field).unwrap();
    let schema = lf.collect_schema().unwrap();

    // Collect all the columns that have an analysis
    let mut names = Vec::new();
    for (name, dtype) in schema.iter() {
        if dtype == &DataType::String {
            names.push(name.to_string());
        }
    }

    // Build an expression for the columns
    let expr = names
        .iter()
        .map(|name| col(name.to_string()))
        .collect::<Vec<_>>();

    lf.select(expr).collect().unwrap()
}

/// Load IPv6 addresses from a text file (one per line)
pub fn load_ipv6_addresses_from_file(file: &PathBuf) -> Result<Vec<[u8; 16]>, String> {
    let file = File::open(file)
        .map_err(|e| format!("Failed to open input file: {}", e))?;
    
    let reader = BufReader::new(file);
    let mut addresses = Vec::new();
    
    for (line_num, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("Failed to read line {}: {}", line_num + 1, e))?;
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse as IPv6 address
        let ip = Ipv6Addr::from_str(line)
            .map_err(|e| format!("Failed to parse IPv6 address on line {}: {}", line_num + 1, e))?;
        
        addresses.push(ip.octets());
    }
    
    if addresses.is_empty() {
        return Err("No valid IPv6 addresses found in input file".to_string());
    }
    
    Ok(addresses)
}