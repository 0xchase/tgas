use std::io::{BufRead, Error as IoError};
use std::mem;
use std::net::IpAddr;

const INITIAL_BUFFER_SIZE: usize = 48;

pub struct IpListIterator<R> {
    reader: R,
    line_buffer: String,
    total_lines: usize,
    bytes_read: u64,
}

impl<R: BufRead> IpListIterator<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            line_buffer: String::new(),
            total_lines: 0,
            bytes_read: 0,
        }
    }

    pub fn bytes_read(&self) -> u64 {
        self.bytes_read
    }
}

impl<R: BufRead> Iterator for IpListIterator<R> {
    type Item = Result<IpAddr, IoError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.line_buffer.clear();
            match self.reader.read_line(&mut self.line_buffer) {
                Ok(0) => return None,
                Ok(n) => {
                    self.total_lines += 1;
                    self.bytes_read += n as u64;
                    let line = self.line_buffer.as_bytes();
                    if line.is_empty() || line[0] == b'#' {
                        continue;
                    }
                    let start = line
                        .iter()
                        .position(|&b| !b.is_ascii_whitespace())
                        .unwrap_or(0);
                    let end = line
                        .iter()
                        .rposition(|&b| !b.is_ascii_whitespace())
                        .unwrap_or(0);
                    if start > end {
                        continue;
                    }
                    let trimmed = unsafe { std::str::from_utf8_unchecked(&line[start..=end]) };
                    match trimmed.parse::<IpAddr>() {
                        Ok(addr) => return Some(Ok(addr)),
                        Err(e) => {
                            return Some(Err(IoError::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Failed to parse IP address '{}': {}", trimmed, e),
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
        (0, None)
    }
}
