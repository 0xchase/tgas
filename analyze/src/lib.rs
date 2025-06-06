use std::io::{BufRead, Error as IoError};
use std::net::{IpAddr, Ipv6Addr};
use std::collections::HashMap;
use std::str::FromStr;
use std::fmt::Display;

mod analysis;
mod formats;

pub use formats::{IpListIterator, ScanResultIterator, ScanResultRow};
pub use analysis::{DispersionAnalysis, EntropyAnalysis, StatisticsAnalysis, SubnetAnalysis};

/// Trait for analysis results that can be printed
pub trait PrintableResults: Display {
    fn print(&self);
}

/// Trait for IPv6 address analysis implementations
pub trait Analysis<T> {
    type Results: PrintableResults;

    /// Absorb a new value into the analysis
    fn absorb(&mut self, value: T);
    
    /// Get the final analysis results
    fn results(self) -> Self::Results;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubnetOptions {
    pub max_subnets: usize,
    pub prefix_length: u8,
}

impl Default for SubnetOptions {
    fn default() -> Self {
        Self {
            max_subnets: 10,
            prefix_length: 64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum AnalysisType {
    /// Basic address counts and statistics (total, unique, duplicates)
    Counts,
    /// Address space dispersion metrics (distances between addresses)
    Dispersion,
    /// Information entropy analysis of address distribution
    Entropy,
    /// Subnet distribution analysis
    Subnets,
}

pub fn analyze<R: BufRead>(reader: R, analysis_type: AnalysisType, subnet_options: Option<SubnetOptions>) -> Result<Box<dyn PrintableResults>, IoError> {
    // First identify the format
    let mut line_buffer = String::new();
    let mut lines = Vec::new();
    {
        let mut peek_reader = reader;
        while peek_reader.read_line(&mut line_buffer)? > 0 {
            lines.push(line_buffer.clone());
            line_buffer.clear();
        }
    }
    
    // Create a new reader from the collected lines
    let content = lines.join("");
    let reader = std::io::Cursor::new(content);
    let format = formats::identify_format(std::io::BufReader::new(reader))?;

    // Create a new reader for actual processing
    let reader = std::io::Cursor::new(lines.join(""));
    let reader = std::io::BufReader::new(reader);

    // Process the input based on format and analysis type
    match (format, analysis_type) {
        (formats::Format::IpList | formats::Format::ScanResult, analysis_type) => {
            match analysis_type {
                AnalysisType::Counts => {
                    let mut analyzer = StatisticsAnalysis::new();
                    process_input(reader, format, &mut analyzer)?;
                    Ok(Box::new(analyzer.results()))
                },
                AnalysisType::Dispersion => {
                    let mut analyzer = DispersionAnalysis::new();
                    process_input(reader, format, &mut analyzer)?;
                    Ok(Box::new(analyzer.results()))
                },
                AnalysisType::Entropy => {
                    let mut analyzer = EntropyAnalysis::new();
                    process_input(reader, format, &mut analyzer)?;
                    Ok(Box::new(analyzer.results()))
                },
                AnalysisType::Subnets => {
                    let options = subnet_options.unwrap_or_default();
                    let mut analyzer = SubnetAnalysis::new_with_options(options.max_subnets, options.prefix_length);
                    process_input(reader, format, &mut analyzer)?;
                    Ok(Box::new(analyzer.results()))
                },
            }
        },
        (formats::Format::Unknown, _) => {
            Err(IoError::new(
                std::io::ErrorKind::InvalidData,
                "Could not determine file format"
            ))
        }
    }
}

fn process_input<R: BufRead, A: Analysis<Ipv6Addr>>(
    reader: R,
    format: formats::Format,
    analyzer: &mut A,
) -> Result<(), IoError> {
    match format {
        formats::Format::IpList => {
            let iter = IpListIterator::new(reader);
            for result in iter {
                match result? {
                    IpAddr::V6(addr) => analyzer.absorb(addr),
                    _ => continue, // Skip non-IPv6 addresses
                }
            }
        }
        formats::Format::ScanResult => {
            let iter = ScanResultIterator::new(reader)?;
            for result in iter {
                let row = result?;
                analyzer.absorb(row.address);
            }
        }
        formats::Format::Unknown => {
            return Err(IoError::new(
                std::io::ErrorKind::InvalidData,
                "Could not determine file format"
            ));
        }
    }
    Ok(())
}

pub fn print_analysis_result(result: &dyn PrintableResults) {
    result.print();
}
