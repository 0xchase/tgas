use std::net::Ipv6Addr;
use polars::prelude::*;
use plugin::contracts::Predicate;
use indicatif::{ProgressBar, ProgressStyle, ParallelProgressIterator};
use rayon::prelude::*;
use crate::analysis::predicates::*;

pub struct FilterAnalysis {
    predicate_name: String,
}

impl FilterAnalysis {
    pub fn new(predicate_name: String) -> Self {
        Self { predicate_name }
    }

    pub fn analyze(&self, series: &Series) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let all_predicates = get_all_predicates();
        let predicate_fn = all_predicates
            .into_iter()
            .find(|(name, _)| name == &self.predicate_name)
            .map(|(_, func)| func)
            .ok_or_else(|| format!("No predicate found with name: {}", self.predicate_name))?;

        // --- 1. Parse IP addresses ---
        let utf8_series = series.str().map_err(|e| format!("Failed to convert to string series: {}", e))?;

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

        // --- 2. Filter addresses by predicate ---
        let filtered_addresses: Vec<String> = parsed_ips
            .into_iter()
            .filter_map(|opt_addr| opt_addr)
            .filter(|addr| predicate_fn(*addr))
            .map(|addr| addr.to_string())
            .collect();

        // --- 3. Create the final DataFrame ---
        let df = DataFrame::new(vec![
            Series::new("address".into(), filtered_addresses).into(),
        ])?;

        Ok(df)
    }
}

pub struct FilterResults {
    pub dataframe: DataFrame,
}

impl FilterResults {
    pub fn new(dataframe: DataFrame) -> Self {
        Self { dataframe }
    }
} 