use std::fs::File;
use std::io::{BufRead, Error as IoError};
use std::net::{Ipv6Addr};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use indicatif::{ProgressBar, ProgressStyle};
use polars::prelude::*;
use plugin::contracts::{AbsorbField, MyField};

use analyze::analysis::{DispersionAnalysis, ShannonEntropyAnalysis, StatisticsAnalysis, SubnetAnalysis, SpecialAnalysis};
use analyze::analysis::{DispersionResults, ShannonEntropyResults, StatisticsResults, SubnetResults};

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

pub fn analyze_file(
    file: &PathBuf,
    field: &Option<String>,
    analysis_type: AnalysisType,
) -> Result<DataFrame, String> {
    let mut lf = open_csv_lazy(file, field)?;
    let schema = lf.collect_schema().unwrap();

    /*let file = File::open(file).unwrap();
    let mut csv_reader = CsvReader::new(file)
        .with_options(CsvReadOptions::default()
            .with_infer_schema_length(Some(100))
            .with_chunk_size(100000));
    
    let mut batches = csv_reader.batched_borrowed().unwrap();

    let mut total_df = DataFrame::default();
    let mut chunks = Vec::new();
    while let Ok(batch) = batches.next_batches(8) {
        if let Some(batch) = batch {
            println!("{}", batch.len());
            chunks.push(batch);
        }
    }*/

    // println!("{}", chunks.len());



    let mut names = Vec::new();

    // Collect all the columns that have an analysis
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

    let df = lf.select(expr).collect().unwrap();

    match analyze(df, analysis_type) {
        Ok(results) => {
            Ok(results)
        },
        Err(e) => Err(format!("Analysis failed: {}", e)),
    }
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
        self.pb.set_message(format!("Processed {} {}", self.count, self.item_type));
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

pub fn analyze(lf: DataFrame, analysis_type: AnalysisType) -> Result<DataFrame, IoError> {
    match analysis_type {
        AnalysisType::Counts => {
            let mut analyzer = StatisticsAnalysis::new();
            analyze_dataframe(lf, &mut analyzer)
        },
        AnalysisType::Dispersion => {
            let mut analyzer = DispersionAnalysis::new();
            analyze_dataframe(lf, &mut analyzer)
        },
        AnalysisType::Entropy { start_bit, end_bit } => {
            let mut analyzer = ShannonEntropyAnalysis::new_with_options(start_bit, end_bit);
            analyze_dataframe(lf, &mut analyzer)
        },
        AnalysisType::Subnets { max_subnets, prefix_length } => {
            let mut analyzer = SubnetAnalysis::new_with_options(max_subnets, prefix_length);
            analyze_dataframe(lf, &mut analyzer)
        },
        AnalysisType::Special => {
            let mut analyzer = SpecialAnalysis::new();
            analyze_dataframe(lf, &mut analyzer)
        },
    }
}

pub fn analyze_dataframe<A: AbsorbField<Ipv6Addr>>(
    df: DataFrame,
    analyzer: &mut A,
) -> Result<DataFrame, IoError>
{
    for series in df.get_columns() {
        analyze_column(series, analyzer, df.height())?;
    }

    Ok(analyzer.finalize())
}

fn analyze_column<A: AbsorbField<Ipv6Addr>>(
    series: &Column,
    analyzer: &mut A,
    total_rows: usize,
) -> Result<(), IoError>
{
    let mut tracker = ProgressTracker::new(total_rows as u64, "addresses");
    for item in series.str().map_err(|e| IoError::new(
        std::io::ErrorKind::InvalidData,
        format!("Failed to convert series to string: {}", e)
    ))? {
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