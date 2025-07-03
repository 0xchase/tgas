use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use clap::{ArgMatches, Parser};

#[derive(Parser, Clone)]
pub struct EntropyCfg {
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

pub struct Entropy {
    cfg: EntropyCfg,
}

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

    pub fn new(cfg: EntropyCfg) -> Self {
        Self { cfg }
    }

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
        Ok(None)
    }
}

register_plugin!(Entropy, EntropyCfg);