use crate::analysis::predicates::*;
use plugin::contracts::AbsorbField;
use polars::prelude::*;
use std::collections::HashMap;
use std::net::Ipv6Addr;

pub struct CountAnalysis {
    predicate_name: Option<String>,
    predicate_counts: HashMap<&'static str, u64>,
    total_addresses: u64,
}

impl CountAnalysis {
    pub fn new(predicate_name: Option<String>) -> Self {
        let all_predicates = get_all_predicates();
        let mut predicate_counts = HashMap::new();
        for (name, _) in &all_predicates {
            predicate_counts.insert(*name, 0);
        }
        Self {
            predicate_name,
            predicate_counts,
            total_addresses: 0,
        }
    }
}

impl AbsorbField<Ipv6Addr> for CountAnalysis {
    type Config = ();

    fn absorb(&mut self, addr: Ipv6Addr) {
        self.total_addresses += 1;
        let all_predicates = get_all_predicates();
        let predicates_to_check = if let Some(ref name) = self.predicate_name {
            all_predicates
                .into_iter()
                .filter(|(pred_name, _)| pred_name == name)
                .collect::<Vec<_>>()
        } else {
            all_predicates
        };
        for (name, predicate_fn) in predicates_to_check {
            if predicate_fn(addr) {
                let count = self.predicate_counts.get_mut(name).unwrap();
                *count += 1;
            }
        }
    }

    fn finalize(&mut self) -> DataFrame {
        let mut predicate_names = Vec::new();
        let mut counts = Vec::new();
        let mut percentages = Vec::new();
        for (name, count) in &self.predicate_counts {
            if *count > 0 {
                predicate_names.push(name.to_string());
                counts.push(*count);
                let percentage = if self.total_addresses > 0 {
                    (*count as f64 / self.total_addresses as f64) * 100.0
                } else {
                    0.0
                };
                percentages.push(percentage);
            }
        }
        let mut pairs: Vec<_> = predicate_names
            .into_iter()
            .zip(counts.into_iter())
            .zip(percentages.into_iter())
            .collect();
        pairs.sort_by(|a, b| (b.0).1.cmp(&(a.0).1));
        let mut predicate_names = Vec::new();
        let mut counts = Vec::new();
        let mut percentages = Vec::new();
        for ((name, count), percentage) in pairs {
            predicate_names.push(name);
            counts.push(count);
            percentages.push(percentage);
        }
        DataFrame::new(vec![
            Column::new("predicate".into(), &predicate_names),
            Column::new("count".into(), &counts),
            Column::new("percentage".into(), &percentages),
        ]).unwrap()
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
