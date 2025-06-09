use std::fs::File;
use std::io::{BufRead, Error as IoError};
use std::net::{Ipv6Addr};
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, Instant};
use analyze::analysis::statistics::StatisticsConfig;
use indicatif::{ProgressBar, ProgressStyle};
use polars::io::mmap::MmapBytesReader;
use polars::prelude::*;
use plugin::contracts::{AbsorbField, MyField};
use futures::stream::{FuturesUnordered, StreamExt, BoxStream};
use futures::Stream;
use rayon::prelude::*;
use rayon::iter::{ParallelIterator, IntoParallelRefIterator};
use indicatif::ParallelProgressIterator;

use analyze::analysis::{DispersionAnalysis, ShannonEntropyAnalysis, StatisticsAnalysis, SubnetAnalysis, SpecialAnalysis};
use analyze::analysis::{DispersionResults, ShannonEntropyResults, StatisticsResults, SubnetResults};
use tokio::stream;
use tokio::task::JoinHandle;

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

pub async fn analyze_file(
    file: &PathBuf,
    field: &Option<String>,
    analysis_type: AnalysisType,
) -> Result<DataFrame, String> {
    let lf = open_csv_lazy(file, field)?;
    tokio::task::spawn_blocking(move | | {
        let df = lf.collect().map_err(|e| format!("Failed to collect dataframe: {}", e))?;

        match analyze(df, analysis_type) {
            Ok(results) => {
                Ok(results)
            },
            Err(e) => Err(format!("Analysis failed: {}", e)),
        }
    }).await.unwrap()
}

fn analyze_column3(s: Column) -> DataFrame {
    let name = s.name().to_string();
    let mean = s.f64().unwrap().mean().unwrap_or(f64::NAN);
    df! { "column" => &[name], "mean" => &[mean] }.unwrap()
}

/// Build a Stream of JoinHandle<DataFrame> for each column in the LazyFrame
async fn column_analysis_handles3(
    lf: LazyFrame
) -> PolarsResult<BoxStream<'static, JoinHandle<DataFrame>>> {
    // 1) Collect on blocking pool
    let df = tokio::task::spawn_blocking(move || lf.collect().unwrap())
        .await
        .unwrap();

    // 2) Hand the Vec<Series> into our generic spawner
    let handles = spawn_tasks_stream(df.get_columns().to_vec(), analyze_column3);
    Ok(handles)
}

fn spawn_tasks_stream<I, T, F>(inputs: I, worker: F) 
    -> BoxStream<'static, JoinHandle<T>>
where
    I: IntoIterator + Send + 'static,
    I::IntoIter: Send + 'static,
    I::Item: Send + 'static,
    T: Send + 'static,
    F: Fn(I::Item) -> T + Send + Sync + 'static + Clone,
{
    futures::stream::iter(inputs.into_iter())
        .map(move |item| {
            // spawn_blocking because we assume `worker` is CPU-bound
            tokio::task::spawn_blocking({
                let worker = worker.clone();
                move || worker(item)
            })
        })
        .boxed()
}

pub fn absorb_series<C, T: MyField, A: AbsorbField<T, Config = C>>(analyzer: &mut A, series: &Series) -> Result<T, IoError> {
    todo!()
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

pub fn analyze(df: DataFrame, analysis_type: AnalysisType) -> Result<DataFrame, IoError> {
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
    df: DataFrame,
    analyzer: &mut A,
) -> Result<DataFrame, IoError>
{
    println!("Collecting schema...");

    for series in df.iter() {
        for item in series.phys_iter() {
            if let Ok(addr) = item.str_value().parse::<Ipv6Addr>() {
                analyzer.absorb(addr);
            }
        }
    }

    Ok(analyzer.finalize())
}

fn analyze_column<A: AbsorbField<Ipv6Addr>>(
    series: &Column,
    analyzer: &mut A,
    total_rows: usize,
) -> Result<(), IoError>
where
    A::Config: Default,
{
    let mut tracker = ProgressTracker::new(total_rows as u64, "addresses");
    
    match series.dtype() {
        DataType::String => {
            for item in series.phys_iter() {
                if let Ok(addr) = item.str_value().parse::<Ipv6Addr>() {
                    analyzer.absorb( addr);
                }
                tracker.increment(tracker.count as u64);
            }
        }
        _ => ()
    }

    tracker.finish(true);
    Ok(())
}