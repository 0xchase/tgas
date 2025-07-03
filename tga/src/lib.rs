mod entropy_ip;
pub mod python_tga;
mod random_ip;

use inventory;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::collections::HashSet;
use std::net::Ipv6Addr;
use std::sync::Once;

pub use entropy_ip::EntropyIpTga;
use plugin::contracts::PluginInfo;
pub use python_tga::PythonTGA;
pub use python_tga::PythonTgaInfo;
pub use python_tga::get_available_python_tga_infos;
pub use random_ip::RandomIpTga;
use serde::{Serialize, de::DeserializeOwned};

pub trait TgaInfo {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
}

#[typetag::serde]
pub trait TGA: Send + Sync {
    fn train<T: IntoIterator<Item = [u8; 16]>>(seeds: T) -> Result<Self, String>
    where
        Self: Sized;
    fn generate(&self) -> [u8; 16];
    fn generate_unique(&self, count: usize) -> Vec<[u8; 16]> {
        const MAX_ATTEMPTS: usize = 1_000_000;
        let mut set = HashSet::new();
        let mut attempts = 0;
        while set.len() < count && attempts < MAX_ATTEMPTS {
            set.insert(self.generate());
            attempts += 1;
        }
        set.into_iter().collect()
    }
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

#[derive(Clone)]
pub struct TgaRegistration {
    pub name: &'static str,
    pub description: &'static str,
    pub train_fn: fn(Vec<[u8; 16]>) -> Box<dyn TGA>,
}

inventory::collect!(TgaRegistration);

use std::sync::Arc;
use std::sync::Mutex;

static DYNAMIC_PYTHON_TGAS_INIT: Once = Once::new();
static DYNAMIC_PYTHON_TGAS: Mutex<Vec<TgaRegistration>> = Mutex::new(Vec::new());

fn get_dynamic_python_tgas() -> Vec<TgaRegistration> {
    DYNAMIC_PYTHON_TGAS_INIT.call_once(|| {
        println!("[DEBUG] Querying Python TGA registry...");
        let python_tga_infos = match python_tga::get_available_python_tga_infos() {
            Ok(list) => list,
            Err(e) => {
                println!("[DEBUG] Error querying Python TGAs: {e}");
                vec![]
            }
        };
        println!("[DEBUG] Python TGAs found: {:?}", python_tga_infos);
        let mut regs = Vec::new();
        for info in python_tga_infos {
            let name = info.name;
            let description = info.description;
            let name_static: &'static str = Box::leak(name.into_boxed_str());
            let desc_static: &'static str = Box::leak(description.into_boxed_str());
            regs.push(TgaRegistration {
                name: name_static,
                description: desc_static,
                train_fn: create_python_tga_train_fn(name_static),
            });
        }
        let mut dynamic_tgas = DYNAMIC_PYTHON_TGAS.lock().unwrap();
        *dynamic_tgas = regs;
    });
    let result = DYNAMIC_PYTHON_TGAS.lock().unwrap().clone();
    println!(
        "[DEBUG] Returning dynamic Python TGAs: {:?}",
        result.iter().map(|r| r.name).collect::<Vec<_>>()
    );
    result
}

fn create_python_tga_train_fn(tga_name: &'static str) -> fn(Vec<[u8; 16]>) -> Box<dyn TGA> {
    match tga_name {
        "lstm_ipv6" => lstm_ipv6_train_fn,
        _ => generic_python_tga_train_fn,
    }
}

fn lstm_ipv6_train_fn(addresses: Vec<[u8; 16]>) -> Box<dyn TGA> {
    let kwargs = serde_json::json!({});
    let python_tga = PythonTGA::train_with_python("lstm_ipv6", addresses, kwargs)
        .expect("Failed to train Python TGA");
    Box::new(python_tga)
}

fn generic_python_tga_train_fn(addresses: Vec<[u8; 16]>) -> Box<dyn TGA> {
    let kwargs = serde_json::json!({});
    let python_tga = PythonTGA::train_with_python("lstm_ipv6", addresses, kwargs)
        .expect("Failed to train Python TGA");
    Box::new(python_tga)
}

pub struct TgaRegistry;

impl TgaRegistry {
    pub fn get_available_tgas() -> Vec<&'static str> {
        let mut names: Vec<&'static str> = inventory::iter::<TgaRegistration>
            .into_iter()
            .map(|reg| reg.name)
            .collect();
        names
    }
    pub fn get_tga_description(name: &str) -> Option<&'static str> {
        inventory::iter::<TgaRegistration>
            .into_iter()
            .find(|reg| reg.name == name)
            .map(|reg| reg.description)
    }
    pub fn train_tga(
        name: &str,
        addresses: Vec<[u8; 16]>,
    ) -> Result<Box<dyn TGA + Sync + Send + 'static>, String> {
        if let Some(reg) = inventory::iter::<TgaRegistration>
            .into_iter()
            .find(|reg| reg.name == name)
        {
            Ok((reg.train_fn)(addresses))
        } else {
            let python_tgas = get_dynamic_python_tgas();
            if let Some(reg) = python_tgas.iter().find(|reg| reg.name == name) {
                Ok((reg.train_fn)(addresses))
            } else {
                Err(format!("Unknown TGA type: {}", name))
            }
        }
    }
    pub fn deserialize_tga(
        model_data: &[u8],
    ) -> Result<Box<dyn TGA + Sync + Send + 'static>, String> {
        bincode::deserialize::<Box<dyn TGA>>(model_data)
            .map(|b| b as Box<dyn TGA + Sync + Send + 'static>)
            .map_err(|e| format!("Failed to deserialize model: {}", e))
    }
    pub fn get_tga_help_text() -> String {
        let mut help = String::from("Type of TGA to train. Available types:\n");
        for reg in inventory::iter::<TgaRegistration> {
            help.push_str(&format!("  {} - {}\n", reg.name, reg.description));
        }
        help
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_inventory_registration() {
        let tgas = TgaRegistry::get_available_tgas();
        assert!(!tgas.is_empty(), "No TGAs registered in inventory");
        assert!(
            tgas.contains(&"entropy_ip"),
            "entropy_ip not found in registry"
        );
        assert!(
            tgas.contains(&"random_ip"),
            "random_ip not found in registry"
        );
        let help_text = TgaRegistry::get_tga_help_text();
        assert!(
            help_text.contains("entropy_ip"),
            "Help text missing entropy_ip"
        );
        assert!(
            help_text.contains("random_ip"),
            "Help text missing random_ip"
        );
        println!("Registered TGAs: {:?}", tgas);
        println!("Help text:\n{}", help_text);
    }
}

pub fn generate(count: usize, unique: bool) {
    let seed_ips: Vec<[u8; 16]> = vec![
        Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0001, 0, 0, 0, 0x0001).octets(),
        Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0001, 0, 0, 0, 0x0002).octets(),
        Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0002, 0, 0, 0, 0x0001).octets(),
        Ipv6Addr::new(0x2001, 0x0db8, 0x0001, 0x0002, 0, 0, 0, 0x0002).octets(),
        Ipv6Addr::new(0x2001, 0x0db8, 0x0002, 0x000a, 0, 0, 0, 0x000a).octets(),
        Ipv6Addr::new(0x2001, 0x0db8, 0x0002, 0x000a, 0, 0, 0, 0x000b).octets(),
        Ipv6Addr::new(0x2001, 0x0db8, 0x0002, 0x000b, 0, 0, 0, 0x000a).octets(),
        Ipv6Addr::new(
            0x2001, 0x0db8, 0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666,
        )
        .octets(),
        Ipv6Addr::new(
            0x2001, 0x0db8, 0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6667,
        )
        .octets(),
    ];

    println!("Building model from {} seed addresses...", seed_ips.len());
    let tga = EntropyIpTga::train(seed_ips).expect("Failed to train model");

    let mut generated = HashSet::new();
    let mut i = 0;
    let mut attempts = 0;
    const MAX_ATTEMPTS: usize = 1_000_000;

    while i < count {
        let generated_bytes = tga.generate();
        let generated_ip = Ipv6Addr::from(generated_bytes);

        if !unique || generated.insert(generated_ip) {
            println!("{}", generated_ip);
            i += 1;
            attempts = 0;
        } else {
            attempts += 1;
            if attempts >= MAX_ATTEMPTS {
                eprintln!(
                    "Warning: Could only generate {}/{} unique addresses after {} attempts",
                    i, count, MAX_ATTEMPTS
                );
                break;
            }
        }
    }
}
