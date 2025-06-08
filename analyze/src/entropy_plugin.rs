use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use arrow_array::{Float64Array, RecordBatch};
use arrow_schema::{DataType, Schema};
use clap::{ArgMatches, Parser};
/*use plugin::{FieldSpec, Plugin, Stage, register_plugin};

/// ① What flags your plugin accepts
#[derive(Parser, Clone)]
pub struct EntropyCfg {
    /// Only look at the last N bytes
    #[arg(long)]
    tail_bytes: Option<usize>,
}

impl EntropyCfg {
    pub fn from_arg_matches(m: &ArgMatches) -> Result<Self> {
        Ok(Self {
            tail_bytes: m.get_one::<usize>("tail-bytes").cloned(),
        })
    }
}

/// ② Plugin struct holds parsed flags
pub struct Entropy {
    cfg: EntropyCfg,
}

/// ③ All metadata & schemas as consts
impl Entropy {
    pub const NAME:    &'static str = "entropy";
    pub const VERSION: &'static str = "0.1.0";
    pub const STAGE:   Stage      = Stage::Analyse;
    pub const ABOUT:   &'static str = "Compute Shannon entropy per IPv6 address";

    pub const INPUT: &'static [FieldSpec] = &[
        FieldSpec::new("addr", DataType::FixedSizeBinary(16), false),
    ];
    pub const OUTPUT: &'static [FieldSpec] = &[
        FieldSpec::new("entropy", DataType::Float64, false),
    ];

    /// Constructor
    pub fn new(cfg: EntropyCfg) -> Self {
        Self { cfg }
    }

    /// entropy helper
    fn entropy(&self, buf: &[u8]) -> f64 {
        let slice = if let Some(n) = self.cfg.tail_bytes {
            let start = buf.len().saturating_sub(n);
            &buf[start..]
        } else {
            buf
        };
        let mut counts = [0u32; 256];
        for &b in slice { counts[b as usize] += 1; }
        let len = slice.len() as f64;
        counts.iter()
              .filter(|&&c| c>0)
              .map(|&c| { let p=c as f64/len; -p*p.log2() })
              .sum()
    }
}

#[async_trait]
impl Plugin for Entropy {
    async fn run(&mut self, batch: RecordBatch) -> Result<Option<RecordBatch>> {
        /*let addr_arr = as_fixed_size_binary_array(
            batch.column_by_name("addr")
                 .expect("INPUT verification")
                 .as_ref(),
        );

        let ent: Vec<f64> = (0..addr_arr.len())
            .map(|i| self.entropy(addr_arr.value(i)))
            .collect();
        let ent_arr = Float64Array::from(ent);

        // stitch columns
        let mut cols = batch.columns().to_vec();
        cols.push(std::sync::Arc::new(ent_arr) as _);

        // derive outgoing schema = incoming.fields + OUTPUT
        let mut fields = batch.schema().fields().to_vec();
        for &spec in Self::OUTPUT {
            fields.push(spec.into());
        }
        let out_schema = Arc::new(Schema::new(fields));

        let out = RecordBatch::try_new(out_schema, cols)?;
        Ok(Some(out))*/
        Ok(None)
    }
}

// ④ register with (PluginType, ConfigType)
register_plugin!(Entropy, EntropyCfg);
*/