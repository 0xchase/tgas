use std::io::{BufRead, Error as IoError};
use std::net::IpAddr;
use std::mem;

const INITIAL_BUFFER_SIZE: usize = 48; // Slightly larger than max IPv6 string (39 chars) + newline

pub struct IpListIterator<R> {
    reader: R,
    line_buffer: String,
    #[allow(dead_code)]
    total_lines: usize,
}

impl<R: BufRead> IpListIterator<R> {
    #[inline]
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            line_buffer: String::with_capacity(INITIAL_BUFFER_SIZE),
            total_lines: 0,
        }
    }

    #[inline]
    pub fn with_capacity(reader: R, capacity: usize) -> Self {
        Self {
            reader,
            line_buffer: String::with_capacity(capacity.max(INITIAL_BUFFER_SIZE)),
            total_lines: 0,
        }
    }
}

impl<R: BufRead> Iterator for IpListIterator<R> {
    type Item = Result<IpAddr, IoError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Clear buffer without deallocating
            self.line_buffer.clear();
            
            // Read next line
            match self.reader.read_line(&mut self.line_buffer) {
                Ok(0) => return None, // EOF
                Ok(_) => {
                    self.total_lines += 1;
                    
                    // Fast path: check if empty or comment without allocating a new string
                    let line = self.line_buffer.as_bytes();
                    if line.is_empty() || line[0] == b'#' {
                        continue;
                    }

                    // Trim in place to avoid allocation
                    let start = line.iter().position(|&b| !b.is_ascii_whitespace()).unwrap_or(0);
                    let end = line.iter().rposition(|&b| !b.is_ascii_whitespace()).unwrap_or(0);
                    if start > end {
                        continue; // Empty line
                    }

                    // Get trimmed slice without allocating
                    let trimmed = unsafe {
                        std::str::from_utf8_unchecked(&line[start..=end])
                    };

                    // Parse IP address
                    match trimmed.parse::<IpAddr>() {
                        Ok(addr) => return Some(Ok(addr)),
                        Err(e) => {
                            // Only allocate string for error case
                            return Some(Err(IoError::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Failed to parse IP address '{}': {}", trimmed, e)
                            )));
                        }
                    }
                }
                Err(e) => return Some(Err(e)),
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // We can't know the exact size without reading the whole file
        (0, None)
    }
} 