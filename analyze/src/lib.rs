use std::io::{BufRead, Error as IoError};
use std::net::{IpAddr, Ipv6Addr};
use std::fmt::Display;
use std::time::{Duration, Instant};
use indicatif::{ProgressBar, ProgressStyle};

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

struct ProgressTracker {
    pb: ProgressBar,
    count: usize,
    bytes_read: u64,
    item_type: &'static str,
    last_update: Instant,
    update_interval: Duration,
}

impl ProgressTracker {
    fn new(total_size: u64, item_type: &'static str) -> Self {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] Processed {msg} [{bar:20.cyan/blue}] {bytes}/{total_bytes}")
                .expect("Failed to create progress bar template")
                // .progress_chars("█░░")
        );
        pb.set_message(format!("0 {}", item_type));

        Self {
            pb,
            count: 0,
            bytes_read: 0,
            item_type,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(50), // 50 updates per second
        }
    }

    fn increment(&mut self, current_bytes: u64) {
        self.count += 1;
        self.bytes_read = current_bytes;

        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.update_interval {
            self.update_progress();
            self.last_update = now;
        }
    }

    fn update_progress(&mut self) {
        self.pb.set_position(self.bytes_read);
        self.pb.set_message(format!("{} {}", self.count, self.item_type));
    }

    fn finish(mut self, success: bool) {
        // Ensure final progress is shown
        self.update_progress();
        
        if success {
            self.pb.finish_with_message("Analysis complete!");
        } else {
            self.pb.abandon_with_message("Analysis failed");
        }
    }
}

pub fn analyze<R: BufRead>(mut reader: R, analysis_type: AnalysisType, subnet_options: Option<SubnetOptions>, file_size: u64) -> Result<Box<dyn PrintableResults>, IoError> {
    // Identify format
    let format = formats::identify_format(&mut reader)?;

    // Process the input based on format and analysis type
    match (format, analysis_type) {
        (formats::Format::IpList | formats::Format::ScanResult, analysis_type) => {
            match analysis_type {
                AnalysisType::Counts => {
                    let mut analyzer = StatisticsAnalysis::new();
                    process_input(reader, format, &mut analyzer, file_size)?;
                    Ok(Box::new(analyzer.results()))
                },
                AnalysisType::Dispersion => {
                    let mut analyzer = DispersionAnalysis::new();
                    process_input(reader, format, &mut analyzer, file_size)?;
                    Ok(Box::new(analyzer.results()))
                },
                AnalysisType::Entropy => {
                    let mut analyzer = EntropyAnalysis::new();
                    process_input(reader, format, &mut analyzer, file_size)?;
                    Ok(Box::new(analyzer.results()))
                },
                AnalysisType::Subnets => {
                    let options = subnet_options.unwrap_or_default();
                    let mut analyzer = SubnetAnalysis::new_with_options(options.max_subnets, options.prefix_length);
                    process_input(reader, format, &mut analyzer, file_size)?;
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
    file_size: u64,
) -> Result<(), IoError> {
    let mut tracker = ProgressTracker::new(
        file_size,
        match format {
            formats::Format::IpList => "addresses",
            formats::Format::ScanResult => "probe responses",
            formats::Format::Unknown => "unknown entries",
        }
    );

    let result = match format {
        formats::Format::IpList => {
            let mut iter = IpListIterator::new(reader);
            while let Some(result) = iter.next() {
                match result? {
                    IpAddr::V6(addr) => {
                        analyzer.absorb(addr);
                        tracker.increment(iter.bytes_read());
                    }
                    _ => continue, // Skip non-IPv6 addresses
                }
            }
            // Update remaining progress
            if tracker.count > 0 {
                tracker.update_progress();
            }
            Ok(())
        }
        formats::Format::ScanResult => {
            let mut iter = ScanResultIterator::new(reader)?;
            while let Some(result) = iter.next() {
                let row = result?;
                analyzer.absorb(row.address);
                tracker.increment(iter.bytes_read());
            }
            // Update remaining progress
            if tracker.count > 0 {
                tracker.update_progress();
            }
            Ok(())
        }
        formats::Format::Unknown => {
            Err(IoError::new(
                std::io::ErrorKind::InvalidData,
                "Could not determine file format"
            ))
        }
    };

    tracker.finish(result.is_ok());
    result
}

pub fn print_analysis_result(result: &dyn PrintableResults) {
    result.print();
}
