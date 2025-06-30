use serde::{Deserialize, Serialize};
use plugin::contracts::PluginInfo;
use crate::TGA;
use rand::Rng;

/// TGA implementation that generates random IPv6 addresses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomIpTga {
    // No state needed for random generation
}

impl PluginInfo for RandomIpTga {
    const NAME: &'static str = "random_ip";
    const DESCRIPTION: &'static str = "Random IPv6 address generation using cryptographically secure random bytes";
}

impl RandomIpTga {
    pub const NAME: &'static str = "random_ip";
    pub const DESCRIPTION: &'static str = "Random IPv6 address generation using cryptographically secure random bytes";
}

#[typetag::serde]
impl TGA for RandomIpTga {
    fn train<T: IntoIterator<Item = [u8; 16]>>(_seeds: T) -> Result<Self, String> {
        // No training needed for random generation
        Ok(RandomIpTga {})
    }

    fn generate(&self) -> [u8; 16] {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes);
        bytes
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn description(&self) -> &'static str {
        Self::DESCRIPTION
    }
}

fn random_ip_train_fn(addresses: Vec<[u8; 16]>) -> Box<dyn crate::TGA> {
    Box::new(<RandomIpTga as crate::TGA>::train(addresses).expect("Training failed"))
}

inventory::submit! {
    crate::TgaRegistration {
        name: RandomIpTga::NAME,
        description: RandomIpTga::DESCRIPTION,
        train_fn: random_ip_train_fn,
    }
} 