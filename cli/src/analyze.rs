use indicatif::{ProgressBar, ProgressStyle};
use plugin::contracts::{AbsorbField, MyField};
use polars::prelude::*;
use std::fs::File;
use std::io::{BufRead, Error as IoError};
use std::net::Ipv6Addr;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use analyze::analysis::{
    CountAnalysis, DispersionAnalysis, ShannonEntropyAnalysis, StatisticsAnalysis, SubnetAnalysis, UniqueAnalysis,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisType {
    /// Basic address counts and statistics (total, unique, duplicates)
    Unique,
    /// Address space dispersion metrics (distances between addresses)
    Dispersion,
    /// Information entropy analysis
    Entropy { start_bit: u8, end_bit: u8 },
    /// Subnet distribution analysis
    Subnets {
        max_subnets: usize,
        prefix_length: u8,
    },
    /// Count addresses matching each predicate
    Counts,
    /// Special IPv6 address block analysis
    Special,
    /// EUI-64 address analysis (extract MAC addresses)
    Eui64,
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
                .template("[{elapsed_precise}] {msg} [{bar:20.cyan/grey}] {bytes}/{total_bytes}")
                .expect("Failed to create progress bar template")
                .progress_chars("█░"),
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
        self.pb
            .set_message(format!("Processed {} {}", self.count, self.item_type));
    }

    fn finish(mut self, success: bool) {
        // Ensure final progress is shown
        self.update_progress();

        if success {
            self.pb.finish_with_message("Processing complete!");
        } else {
            self.pb.abandon_with_message("Processing failed");
        }
    }
}

pub fn analyze(df: DataFrame, analysis_type: AnalysisType) -> Result<DataFrame, IoError> {
    match analysis_type {
        AnalysisType::Unique => {
            // For Unique analysis, return the first series result
            if let Some(series) = df.get_columns().first() {
                let analyzer = UniqueAnalysis::new(None);
                analyzer
                    .analyze(series.as_series().unwrap())
                    .map_err(|e| IoError::new(std::io::ErrorKind::InvalidData, e.to_string()))
            } else {
                Err(IoError::new(
                    std::io::ErrorKind::InvalidData,
                    "No data to analyze",
                ))
            }
        }
        AnalysisType::Dispersion => {
            // For Dispersion analysis, return the first series result
            if let Some(series) = df.get_columns().first() {
                let mut analyzer = DispersionAnalysis::new();
                analyze_column(series, &mut analyzer, df.height())?;
                let output = analyzer.finalize();
                Ok(output)
            } else {
                Err(IoError::new(
                    std::io::ErrorKind::InvalidData,
                    "No data to analyze",
                ))
            }
        }
        AnalysisType::Entropy { start_bit, end_bit } => {
            // For Entropy analysis, return the first series result
            if let Some(series) = df.get_columns().first() {
                let mut analyzer = ShannonEntropyAnalysis::new_with_options(start_bit, end_bit);
                analyze_column(series, &mut analyzer, df.height())?;
                let output = analyzer.finalize();
                Ok(output)
            } else {
                Err(IoError::new(
                    std::io::ErrorKind::InvalidData,
                    "No data to analyze",
                ))
            }
        }
        AnalysisType::Subnets {
            max_subnets,
            prefix_length,
        } => {
            // For Subnets analysis, return the first series result
            if let Some(series) = df.get_columns().first() {
                let mut analyzer = SubnetAnalysis::new_with_options(max_subnets, prefix_length);
                analyze_column(series, &mut analyzer, df.height())?;
                let output = analyzer.finalize();
                Ok(output)
            } else {
                Err(IoError::new(
                    std::io::ErrorKind::InvalidData,
                    "No data to analyze",
                ))
            }
        }
        AnalysisType::Counts => {
            // For Counts analysis, return the first series result
            if let Some(series) = df.get_columns().first() {
                let mut analyzer = CountAnalysis::new(None);
                analyze_column(series, &mut analyzer, df.height())?;
                let output = analyzer.finalize();
                Ok(output)
            } else {
                Err(IoError::new(
                    std::io::ErrorKind::InvalidData,
                    "No data to analyze",
                ))
            }
        }
        AnalysisType::Special => Err(IoError::new(
            std::io::ErrorKind::Unsupported,
            "Special analysis not yet implemented",
        )),
        AnalysisType::Eui64 => Err(IoError::new(
            std::io::ErrorKind::Unsupported,
            "EUI-64 analysis not yet implemented",
        )),
    }
}

fn analyze_column<A: AbsorbField<Ipv6Addr>>(
    series: &Column,
    analyzer: &mut A,
    total_rows: usize,
) -> Result<(), IoError> {
    let mut tracker = ProgressTracker::new(total_rows as u64, "addresses");
    for item in series.str().map_err(|e| {
        IoError::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to convert series to string: {}", e),
        )
    })? {
        if let Some(addr_str) = item {
            if let Ok(addr) = addr_str.parse::<Ipv6Addr>() {
                analyzer.absorb(addr);
            }

            tracker.increment(tracker.count as u64);
        }
    }

    tracker.finish(true);
    Ok(())
}


