use std::io::{BufRead, Error as IoError};
use std::net::Ipv6Addr;

#[derive(Debug)]
pub struct ScanResultRow {
    pub address: Ipv6Addr,
    pub is_active: bool,
    pub raw_fields: Vec<String>,
}

pub struct ScanResultIterator<R> {
    reader: R,
    line_buffer: String,
    bytes_read: u64,
    saddr_idx: usize,
    type_idx: Option<usize>,
    header_read: bool,
}

impl<R: BufRead> ScanResultIterator<R> {
    pub fn new(reader: R) -> Result<Self, IoError> {
        let mut iter = Self {
            reader,
            line_buffer: String::new(),
            bytes_read: 0,
            saddr_idx: 0,
            type_idx: None,
            header_read: false,
        };

        iter.line_buffer.clear();
        match iter.reader.read_line(&mut iter.line_buffer) {
            Ok(0) => return Err(IoError::from(std::io::ErrorKind::UnexpectedEof)),
            Ok(_) => {
                let header = iter.line_buffer.trim();
                let columns: Vec<&str> = header.split(',').collect();
                let saddr_idx = columns.iter().position(|&c| c == "saddr").ok_or_else(|| {
                    IoError::new(
                        std::io::ErrorKind::InvalidData,
                        "No saddr column in CSV header",
                    )
                })?;
                let type_idx = columns.iter().position(|&c| c == "type");

                iter.saddr_idx = saddr_idx;
                iter.type_idx = type_idx;
                iter.header_read = true;
                Ok(iter)
            }
            Err(e) => Err(e),
        }
    }

    #[inline]
    pub fn bytes_read(&self) -> u64 {
        self.bytes_read
    }
}

impl<R: BufRead> Iterator for ScanResultIterator<R> {
    type Item = Result<ScanResultRow, IoError>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.header_read {
            return None;
        }

        loop {
            self.line_buffer.clear();
            match self.reader.read_line(&mut self.line_buffer) {
                Ok(0) => return None,
                Ok(n) => {
                    self.bytes_read += n as u64;
                    let line = self.line_buffer.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    let fields: Vec<String> =
                        line.split(',').map(|s| s.trim().to_string()).collect();

                    if fields.len() <= self.saddr_idx {
                        return Some(Err(IoError::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Row has fewer fields than expected: {}", line),
                        )));
                    }

                    match fields[self.saddr_idx].parse::<Ipv6Addr>() {
                        Ok(addr) => {
                            let is_active = self
                                .type_idx
                                .map(|idx| fields.get(idx).map_or(false, |t| t == "129"))
                                .unwrap_or(false);

                            return Some(Ok(ScanResultRow {
                                address: addr,
                                is_active,
                                raw_fields: fields,
                            }));
                        }
                        Err(e) => {
                            return Some(Err(IoError::new(
                                std::io::ErrorKind::InvalidData,
                                format!(
                                    "Failed to parse IPv6 address '{}': {}",
                                    fields[self.saddr_idx], e
                                ),
                            )));
                        }
                    }
                }
                Err(e) => return Some(Err(e)),
            }
        }
    }
}
