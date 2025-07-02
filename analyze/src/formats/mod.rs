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
    if reader.read_line(&mut first_line)? == 0 {
        return Err(IoError::new(
            std::io::ErrorKind::InvalidData,
            "File is empty",
        ));
    }

    let trimmed = first_line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        // Skip empty lines and comments until we find content
        while reader.read_line(&mut first_line)? > 0 {
            let trimmed = first_line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
            first_line.clear();
        }
    }

    let trimmed = first_line.trim();
    // Determine format based on content
    Ok(
        if trimmed.contains(',') && trimmed.to_lowercase().contains("saddr") {
            Format::ScanResult
        } else if trimmed.parse::<IpAddr>().is_ok() {
            Format::IpList
        } else if trimmed.contains(',') {
            Format::ScanResult
        } else {
            Format::Unknown
        },
    )
}
