use std::net::Ipv6Addr;
use polars::prelude::*;
use plugin::contracts::Predicate;
use tracing::{info, warn, span, Level};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use crate::analysis::predicates::*;

pub struct UniqueAnalysis {
    predicate_name: Option<String>,
}

impl UniqueAnalysis {
    pub fn new(predicate_name: Option<String>) -> Self {
        Self { predicate_name }
    }

    pub fn analyze(&self, series: &Series) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let span = span!(Level::INFO, "unique_analysis", 
            predicate_name = self.predicate_name.as_deref().unwrap_or("all"));
        let _enter = span.enter();

        // Configure rayon to use up to 8 threads
        rayon::ThreadPoolBuilder::new()
            .num_threads(8)
            .build_global()
            .unwrap_or_else(|_| {
                // If global pool is already initialized, just continue
                warn!("Could not set thread pool size (may already be initialized)");
            });

        let all_predicates = get_all_predicates();
        let predicates_to_run = if let Some(ref name) = self.predicate_name {
            all_predicates
                .into_iter()
                .filter(|(pred_name, _)| pred_name == name)
                .collect::<Vec<_>>()
        } else {
            all_predicates
        };

        if predicates_to_run.is_empty() {
            let err_msg = self.predicate_name.as_ref()
                .map_or("No predicates available.".to_string(), |name| format!("No predicate found with name: {}", name));
            return Err(err_msg.into());
        }

        // --- 2. Vectorized Parsing (Do this only ONCE) ---
        // Cast the generic Series to its specific Utf8 type
        let utf8_series = series.str().map_err(|e| format!("Failed to convert to string series: {}", e))?;

        info!("Starting IPv6 address parsing for {} addresses", utf8_series.len());

        // Create progress bar for parsing phase
        let parse_pb = ProgressBar::new(utf8_series.len() as u64);
        parse_pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {msg} [{bar:20.cyan/grey}] {pos}/{len}")
                .expect("Failed to create progress bar template")
                .progress_chars("█░")
        );
        parse_pb.set_message("Parsing IPv6 addresses...");

        // Parse IP addresses and collect results
        let mut parsed_ips = Vec::new();
        for (i, opt_str) in utf8_series.into_iter().enumerate() {
            if let Some(s) = opt_str {
                if let Ok(addr) = s.parse::<Ipv6Addr>() {
                    parsed_ips.push(Some(addr));
                } else {
                    parsed_ips.push(None);
                }
            } else {
                parsed_ips.push(None);
            }
            
            // Update progress every 1000 items to avoid performance impact
            if i % 1000 == 0 {
                parse_pb.set_position(i as u64);
            }
        }
        parse_pb.finish_with_message("IP address parsing complete!");
        info!("IP address parsing complete!");

        // Filter out None values and get unique addresses
        let unique_addresses: std::collections::HashSet<Ipv6Addr> = parsed_ips
            .into_iter()
            .filter_map(|opt| opt)
            .collect();

        info!("Found {} unique IPv6 addresses", unique_addresses.len());

        if unique_addresses.is_empty() {
            // Handle case where no IPs could be parsed.
            return Ok(DataFrame::new(vec![
                Series::new("address".into(), &[] as &[String]).into(),
            ])?);
        }

        // Convert unique addresses to strings for output
        let address_strings: Vec<String> = unique_addresses
            .into_iter()
            .map(|addr| addr.to_string())
            .collect();

        // Create the final DataFrame with unique addresses
        let df = DataFrame::new(vec![
            Series::new("address".into(), address_strings).into(),
        ])?;

        info!("Unique analysis complete");
        Ok(df)
    }
}

pub struct UniqueResults {
    pub dataframe: DataFrame,
}

impl UniqueResults {
    pub fn new(dataframe: DataFrame) -> Self {
        Self { dataframe }
    }
} 