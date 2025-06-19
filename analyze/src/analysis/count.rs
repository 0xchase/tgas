use std::net::Ipv6Addr;
use polars::prelude::*;
use plugin::contracts::Predicate;
use indicatif::{ProgressBar, ProgressStyle, ParallelProgressIterator};
use rayon::prelude::*;
use crate::analysis::predicates::*;

pub struct CountAnalysis {
    predicate_name: Option<String>,
}

impl CountAnalysis {
    pub fn new(predicate_name: Option<String>) -> Self {
        Self { predicate_name }
    }

    pub fn analyze(&self, series: &Series) -> Result<DataFrame, Box<dyn std::error::Error>> {
        // Configure rayon to use up to 8 threads
        rayon::ThreadPoolBuilder::new()
            .num_threads(8)
            .build_global()
            .unwrap_or_else(|_| {
                // If global pool is already initialized, just continue
                eprintln!("Warning: Could not set thread pool size (may already be initialized)");
            });

        let all_predicates = self.get_all_predicates();
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

        // The total is the number of non-null (i.e., successfully parsed) addresses.
        let total = parsed_ips.iter().filter(|opt| opt.is_some()).count() as i64;
        if total == 0 {
            // Handle case where no IPs could be parsed.
            return Ok(DataFrame::new(vec![
                Series::new("predicate".into(), &[] as &[String]).into(),
                Series::new("count".into(), &[] as &[i64]).into(),
                Series::new("total".into(), &[] as &[i64]).into(),
                Series::new("percentage".into(), &[] as &[f64]).into(),
            ])?);
        }

        // --- 3. Apply Predicates and Aggregate Results (Parallel) ---
        // Create progress bar for predicate evaluation
        let eval_pb = ProgressBar::new(predicates_to_run.len() as u64);
        eval_pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {msg} [{bar:20.cyan/grey}] {pos}/{len}")
                .expect("Failed to create progress bar template")
                .progress_chars("█░")
        );
        eval_pb.set_message("Evaluating predicates...");

        // Convert parsed_ips to Arc for sharing across threads
        let parsed_ips_arc = std::sync::Arc::new(parsed_ips);
        
        let results: Vec<(String, i64, i64, f64)> = predicates_to_run
            .par_iter()
            .progress_with(eval_pb)
            .map(|(name, predicate_fn)| {
                let count = parsed_ips_arc.iter()
                    .filter_map(|opt_addr| opt_addr.as_ref())
                    .filter(|addr| predicate_fn(**addr))
                    .count() as i64;

                (
                    name.to_string(),
                    count,
                    total,
                    if total > 0 { (count as f64 / total as f64) * 100.0 } else { 0.0 }
                )
            })
            .collect();

        // Progress bar is finished automatically

        // --- 4. Create the final DataFrame ---
        let names: Vec<String> = results.iter().map(|(name, _, _, _)| name.clone()).collect();
        let counts: Vec<i64> = results.iter().map(|(_, count, _, _)| *count).collect();
        let percentages: Vec<f64> = results.iter().map(|(_, _, _, pct)| *pct).collect();

        let mut df = DataFrame::new(vec![
            Series::new("predicate".into(), names).into(),
            Series::new("count".into(), counts).into(),
            Series::new("percentage".into(), percentages).into(),
        ])?;

        // Filter out rows with count of 0 and sort by count in descending order
        df = df.lazy()
            .filter(col("count").gt(0))
            .sort_by_exprs([col("count")], SortMultipleOptions::new().with_order_descending(true))
            .collect()?;

        Ok(df)
    }

    fn get_all_predicates(&self) -> Vec<(&'static str, fn(Ipv6Addr) -> bool)> {
        vec![
            // Reserved predicates
            ("loopback", |addr| reserved::LoopbackPredicate.predicate(addr)),
            ("unspecified", |addr| reserved::UnspecifiedPredicate.predicate(addr)),
            ("link_local", |addr| reserved::LinkLocalPredicate.predicate(addr)),
            ("unique_local", |addr| reserved::UniqueLocalPredicate.predicate(addr)),
            // ("is_globally_routable_predicate", |addr| reserved::IsGloballyRoutablePredicate.predicate(addr)),
            
            // Multicast predicates
            ("multicast", |addr| multicast::IsMulticastPredicate.predicate(addr)),
            ("solicited_node", |addr| multicast::SolicitedNodeMulticastPredicate.predicate(addr)),
            
            // Transition predicates
            ("ipv4_mapped", |addr| transition::Ipv4MappedPredicate.predicate(addr)),
            ("ipv4_to_ipv6", |addr| transition::Ipv4ToIpv6Predicate.predicate(addr)),
            ("extended_ipv4", |addr| transition::ExtendedIpv4Ipv6Predicate.predicate(addr)),
            ("ipv6_to_ipv4", |addr| transition::Ipv6ToIpv4Predicate.predicate(addr)),
            
            // Documentation predicates
            ("documentation", |addr| documentation::DocumentationPredicate.predicate(addr)),
            ("documentation_2", |addr| documentation::Documentation2Predicate.predicate(addr)),
            ("benchmarking", |addr| documentation::BenchmarkingPredicate.predicate(addr)),
            
            // Protocol predicates
            ("teredo", |addr| protocols::TeredoPredicate.predicate(addr)),
            ("ietf_protocol", |addr| protocols::IetfProtocolPredicate.predicate(addr)),
            ("port_control", |addr| protocols::PortControlProtocolPredicate.predicate(addr)),
            ("turn", |addr| protocols::TurnPredicate.predicate(addr)),
            ("dns_sd", |addr| protocols::DnsSdPredicate.predicate(addr)),
            ("amt", |addr| protocols::AmtPredicate.predicate(addr)),
            ("segment_routing", |addr| protocols::SegmentRoutingPredicate.predicate(addr)),
            
            // Special purpose predicates
            ("discard_only", |addr| special_purpose::DiscardOnlyPredicate.predicate(addr)),
            ("dummy_prefix", |addr| special_purpose::DummyPrefixPredicate.predicate(addr)),
            ("as112_v6", |addr| special_purpose::As112V6Predicate.predicate(addr)),
            ("direct_as112", |addr| special_purpose::DirectAs112Predicate.predicate(addr)),
            ("deprecated_orchid", |addr| special_purpose::DeprecatedOrchidPredicate.predicate(addr)),
            ("orchid_v2", |addr| special_purpose::OrchidV2Predicate.predicate(addr)),
            ("drone_remote_id", |addr| special_purpose::DroneRemoteIdPredicate.predicate(addr)),
            
            // EUI-64 predicates
            ("eui64", |addr| eui64::Eui64Analysis.predicate(addr)),
            ("low_byte_host", |addr| eui64::IsLowByteHostPredicate.predicate(addr)),
            // ("is_privacy_extension_predicate", |addr| eui64::IsPrivacyExtensionPredicate.predicate(addr)),
        ]
    }
}

pub struct CountResults {
    pub dataframe: DataFrame,
}

impl CountResults {
    pub fn new(dataframe: DataFrame) -> Self {
        Self { dataframe }
    }
}
