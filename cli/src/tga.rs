use std::path::PathBuf;
use std::net::Ipv6Addr;
use tga::TGA;
use crate::source;
use bincode;

/// Helper function to get available TGA names for help text
pub fn get_available_tga_names() -> Vec<&'static str> {
    tga::TgaRegistry::get_available_tgas()
}

/// Helper function to get all available TGA names including Python TGAs
pub fn get_all_available_tga_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = vec!["entropy_ip", "random_ip"];
    
    // Add Python TGAs
    match tga::get_available_python_tga_infos() {
        Ok(python_tgas) => {
            for tga_info in python_tgas {
                // Box the name to get 'static lifetime
                let name_static: &'static str = Box::leak(tga_info.name.into_boxed_str());
                names.push(name_static);
            }
        }
        Err(e) => {
            eprintln!("Warning: Could not query Python TGAs: {}", e);
        }
    }
    
    names
}

/// Helper function to train a TGA by name using the registry
pub fn train_tga_by_name(name: &str, input_file: &PathBuf) -> Result<Box<dyn TGA + Send + Sync>, String> {
    // Load addresses from file using source module
    let addresses = source::load_ipv6_addresses_from_file(input_file)?;
    
    // Train the TGA using the registry
    tga::TgaRegistry::train_tga(name, addresses)
}

/// Helper function to generate addresses from a trained TGA model
pub fn generate_tga(model_file: &PathBuf, count: usize, unique: bool) -> Result<(), String> {
    // Load and deserialize the model
    let model_data = std::fs::read(model_file)
        .map_err(|e| format!("Failed to read model file: {}", e))?;
    
    // Deserialize using the registry
    let trained_model = tga::TgaRegistry::deserialize_tga(&model_data)?;
    
    println!("Generating {} addresses{} using {}", count, if unique { " (unique)" } else { "" }, trained_model.name());
    
    if unique {
        let addresses = trained_model.generate_unique(count);
        for addr_bytes in addresses {
            let ip = Ipv6Addr::from(addr_bytes);
            println!("{}", ip);
        }
    } else {
        for _ in 0..count {
            let addr_bytes = trained_model.generate();
            let ip = Ipv6Addr::from(addr_bytes);
            println!("{}", ip);
        }
    }
    
    Ok(())
} 