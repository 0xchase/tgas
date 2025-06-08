use rand::distributions::{Distribution, WeightedIndex};
use std::collections::HashMap;

use crate::IpNetModel;
use crate::TGA;

//- DATA STRUCTURES -----------------------------------------------------------------

/// Represents a mined value within a segment, containing the value and its probability.
#[derive(Debug, Clone)]
pub struct SegmentValue {
    pub value: u128,
    pub probability: f64,
}

/// Represents a segment of an IP address as discovered by the algorithm. 
#[derive(Debug, Clone)]
pub struct Segment {
    pub start_nybble: usize,
    pub end_nybble: usize,
    /// A vector of popular values found in this segment and their associated probabilities.
    pub values: Vec<SegmentValue>,
}

/// The model of the IP network, containing the learned segments.
/// This struct implements the IpNetModel trait for generating new addresses.
#[derive(Debug)]
pub struct EntropyIpModel<const C: usize> {
    pub segments: Vec<Segment>,
}

/// The main struct that implements the TGA trait for the Entropy/IP algorithm.
pub struct EntropyIpTga;

//- IMPLEMENTATION OF IPNETMODEL ----------------------------------------------------

impl<const C: usize> IpNetModel<C> for EntropyIpModel<C> {
    /// Generates a new candidate address based on the probabilistic model. 
    fn generate(&self) -> [u8; C] {
        let mut rng = rand::thread_rng();
        let mut new_address: u128 = 0;

        // Iterate through each learned segment to build the new address piece by piece.
        for segment in &self.segments {
            // Create a weighted distribution based on the mined probabilities for the segment's values.
            let probabilities: Vec<f64> = segment.values.iter().map(|v| v.probability).collect();
            let Ok(dist) = WeightedIndex::new(&probabilities) else {
                // If the segment has no values or probabilities are invalid, we can't generate.
                // In a real scenario, might insert a random or default value.
                continue;
            };

            // Sample from the distribution to pick a value for this segment.
            let chosen_index = dist.sample(&mut rng);
            let chosen_value = segment.values[chosen_index].value;

            // Position the chosen value correctly in the 128-bit address.
            let num_nybbles_in_segment = segment.end_nybble - segment.start_nybble + 1;
            let total_nybbles = C * 2; // Total number of nybbles
            let shift = (total_nybbles - segment.end_nybble - 1) * 4;

            // Clear the bits in the address for this segment before setting them.
            let mask = (1u128 << (num_nybbles_in_segment * 4)) - 1;
            new_address &= !(mask << shift);
            // Set the new value.
            new_address |= chosen_value << shift;
        }

        // Convert the u128 address to a byte array of size C
        let bytes = new_address.to_be_bytes();
        let mut result = [0u8; C];
        result.copy_from_slice(&bytes[16-C..]);
        result
    }
}

//- IMPLEMENTATION OF TGA ----------------------------------------------------------

impl<const C: usize> TGA<C> for EntropyIpTga {
    type Model = EntropyIpModel<C>;

    /// Builds the address model from a list of seed IPs. 
    /// This function orchestrates the main steps of the Entropy/IP algorithm.
    fn build<T: IntoIterator<Item = [u8; C]>>(&self, seeds: T) -> Self::Model {
        let addresses: Vec<u128> = seeds
            .into_iter()
            .map(|bytes| {
                let mut padded = [0u8; 16];
                padded[16-C..].copy_from_slice(&bytes);
                u128::from_be_bytes(padded)
            })
            .collect();

        if addresses.is_empty() {
            return EntropyIpModel { segments: vec![] };
        }

        // STEP 1: Entropy Analysis 
        let entropies = self.calculate_entropies(&addresses);

        // STEP 2: Address Segmentation 
        let mut segments = self.segment_addresses(&entropies, C);

        // STEP 3: Segment Mining 
        self.mine_segments(&mut segments, &addresses);

        // The result is the model containing the learned segments and their probabilities.
        Self::Model { segments }
    }
}

//- HELPER METHODS FOR ENTROPYIPTGA ------------------------------------------------

impl EntropyIpTga {
    /// STEP 1: Calculates the normalized Shannon entropy for each 4-bit nybble. 
    fn calculate_entropies(&self, addresses: &[u128]) -> Vec<f64> {
        let mut entropies = Vec::with_capacity(32);
        let num_addresses = addresses.len() as f64;

        for i in 0..32 {
            let mut counts = HashMap::new();
            for &addr in addresses {
                // Isolate the i-th nybble (from left, 0-indexed).
                let nybble = (addr >> ((31 - i) * 4)) & 0xF;
                *counts.entry(nybble).or_insert(0u64) += 1;
            }

            let mut entropy = 0.0;
            for &count in counts.values() {
                let p = count as f64 / num_addresses;
                if p > 0.0 {
                    entropy -= p * p.log2();
                }
            }
            // Normalize entropy. Max entropy for 16 states (0-F) is log2(16) = 4. 
            entropies.push(entropy / 4.0);
        }
        entropies
    }

    /// STEP 2: Segments addresses based on changes in entropy. 
    fn segment_addresses(&self, entropies: &[f64], const_c: usize) -> Vec<Segment> {
        let mut segments = Vec::new();
        let total_nybbles = const_c * 2;
        if total_nybbles == 0 {
            return segments;
        }

        // Parameters from the paper 
        let thresholds = [0.025, 0.1, 0.3, 0.5, 0.9];
        let hysteresis = 0.05;

        let mut current_segment_start = 0;

        // Rule: Always make bits 1-32 (nybbles 0-7) a separate segment for the /32 prefix. 
        segments.push(Segment {
            start_nybble: 0,
            end_nybble: 7,
            values: Vec::new(),
        });
        current_segment_start = 8;

        // Iterate through remaining nybbles to find other segment boundaries.
        for i in (current_segment_start + 1)..total_nybbles {
            // Rule: Always put a boundary after the 64th bit (nybble 15). 
            if i == 16 {
                segments.push(Segment {
                    start_nybble: current_segment_start,
                    end_nybble: i - 1,
                    values: Vec::new(),
                });
                current_segment_start = i;
                continue;
            }

            let h_prev = entropies[i - 1];
            let h_curr = entropies[i];

            // A new segment starts if entropy crosses a threshold...
            let crosses_threshold = thresholds
                .iter()
                .any(|&t| (h_prev < t && h_curr >= t) || (h_prev >= t && h_curr < t));

            // ...and the change is significant enough (hysteresis). 
            if crosses_threshold && (h_curr - h_prev).abs() > hysteresis {
                segments.push(Segment {
                    start_nybble: current_segment_start,
                    end_nybble: i - 1,
                    values: Vec::new(),
                });
                current_segment_start = i;
            }
        }

        // Add the final segment.
        segments.push(Segment {
            start_nybble: current_segment_start,
            end_nybble: total_nybbles - 1,
            values: Vec::new(),
        });

        segments
    }

    /// STEP 3: Mines popular values and their frequencies for each segment. 
    fn mine_segments(&self, segments: &mut [Segment], addresses: &[u128]) {
        let total_addresses = addresses.len() as f64;

        for segment in segments.iter_mut() {
            let mut value_counts = HashMap::new();
            let num_nybbles_in_segment = segment.end_nybble - segment.start_nybble + 1;
            let shift = (32 - segment.end_nybble - 1) * 4;
            let mask = (1u128 << (num_nybbles_in_segment * 4)) - 1;

            // Extract and count the integer value of this segment for each address.
            for &addr in addresses {
                let value = (addr >> shift) & mask;
                *value_counts.entry(value).or_insert(0) += 1;
            }

            // Convert counts to probabilities and store them.
            // This is a simplified version of the paper's mining, focusing on frequent values. 
            segment.values = value_counts
                .into_iter()
                .map(|(value, count)| SegmentValue {
                    value,
                    probability: count as f64 / total_addresses,
                })
                .collect();
        }
    }
}
