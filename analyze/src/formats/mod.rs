use std::io::{BufRead, Error as IoError};
use std::net::IpAddr;

mod ip_list;
mod scan_result;

pub use ip_list::IpListIterator;
pub use scan_result::{ScanResultIterator, ScanResultRow};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    IpList,
    ScanResult,
    Unknown,
}

/// Identifies the format of the input by examining its content.
/// Returns Format::Unknown if the format cannot be confidently determined.
pub fn identify_format<R: BufRead>(mut reader: R) -> Result<Format, IoError> {
    let mut first_line = String::new();
    
    // Skip empty lines and comments until we find content
    while reader.read_line(&mut first_line)? > 0 {
        let trimmed = first_line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            // If the line contains a comma and "saddr", it's likely a scan result
            if trimmed.contains(',') && trimmed.to_lowercase().contains("saddr") {
                return Ok(Format::ScanResult);
            }
            
            // Try to parse as IP address
            if trimmed.parse::<IpAddr>().is_ok() {
                return Ok(Format::IpList);
            }
            
            // If we can't parse as IP but it has commas, assume it's a scan result
            if trimmed.contains(',') {
                return Ok(Format::ScanResult);
            }
            
            // If we can't confidently determine the format, return Unknown
            return Ok(Format::Unknown);
        }
        first_line.clear();
    }
    
    // If we reach here, the file was empty or only had comments
    Err(IoError::new(
        std::io::ErrorKind::InvalidData,
        "File is empty or contains only comments"
    ))
} 