use std::io::{BufRead, Error as IoError};
use std::net::{Ipv6Addr};
use std::fmt::Display;
use std::time::{Duration, Instant};
use indicatif::{ProgressBar, ProgressStyle};
use polars::prelude::*;
use plugin::contracts::AbsorbField;

mod entropy_plugin;

mod analysis;
mod formats;

pub use formats::{IpListIterator, ScanResultIterator, ScanResultRow};
pub use analysis::{DispersionAnalysis, ShannonEntropyAnalysis, StatisticsAnalysis, SubnetAnalysis};
pub use analysis::{DispersionResults, ShannonEntropyResults, StatisticsResults, SubnetResults};

use crate::analysis::SpecialAnalysis;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisType {
    /// Basic address counts and statistics (total, unique, duplicates)
    Counts,
    /// Address space dispersion metrics (distances between addresses)
    Dispersion,
    /// Information entropy analysis
    Entropy {
        start_bit: u8,
        end_bit: u8,
    },
    /// Subnet distribution analysis
    Subnets {
        max_subnets: usize,
        prefix_length: u8,
    },
    /// Special IPv6 address block analysis
    Special,
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
                .template("[{elapsed_precise}] Processed {msg} [{bar:20.cyan/grey}] {bytes}/{total_bytes}")
                .expect("Failed to create progress bar template")
                .progress_chars("█░")
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

pub fn analyze(df: LazyFrame, analysis_type: AnalysisType) -> Result<DataFrame, IoError> {
    match analysis_type {
        AnalysisType::Counts => {
            let mut analyzer = StatisticsAnalysis::new();
            analyze_dataframe(df, &mut analyzer)
        },
        AnalysisType::Dispersion => {
            let mut analyzer = DispersionAnalysis::new();
            analyze_dataframe(df, &mut analyzer)
        },
        AnalysisType::Entropy { start_bit, end_bit } => {
            let mut analyzer = ShannonEntropyAnalysis::new_with_options(start_bit, end_bit);
            analyze_dataframe(df, &mut analyzer)
        },
        AnalysisType::Subnets { max_subnets, prefix_length } => {
            let mut analyzer = SubnetAnalysis::new_with_options(max_subnets, prefix_length);
            analyze_dataframe(df, &mut analyzer)
        },
        AnalysisType::Special => {
            let mut analyzer = SpecialAnalysis::new();
            analyze_dataframe(df, &mut analyzer)
        },
    }
}

pub fn analyze_dataframe<A: AbsorbField<Ipv6Addr>>(
    df: LazyFrame,
    analyzer: &mut A,
) -> Result<DataFrame, IoError>
where
    A::Config: Default,
{
    let df = df.collect().map_err(|e| IoError::new(
        std::io::ErrorKind::InvalidData,
        format!("Failed to collect DataFrame: {}", e)
    ))?;

    let total_rows = df.height();
    let mut tracker = ProgressTracker::new(total_rows as u64, "addresses");

    for (col_name, series) in df.get_columns().iter().enumerate() {
        // println!("\nAnalyzing column: {}", col_name);
        for value in series.str().map_err(|e| IoError::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to convert series to string: {}", e)
        ))? {
            if let Some(addr_str) = value {
                if let Ok(addr) = addr_str.parse::<Ipv6Addr>() {
                    analyzer.absorb(&A::Config::default(), addr);
                }
            }
            tracker.increment(tracker.count as u64);
        }
    }

    tracker.finish(true);
    Ok(analyzer.finalize())
}
