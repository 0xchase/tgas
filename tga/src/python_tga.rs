use crate::TGA;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Once;

#[derive(Debug, Clone)]
pub struct PythonTgaInfo {
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub struct PythonTGA {
    tga_name: String,
    model_info: Option<Value>,
}

impl PythonTGA {
    pub fn new(tga_name: String) -> Self {
        Self {
            tga_name,
            model_info: None,
        }
    }

    pub fn train_with_python(
        tga_name: &str,
        addresses: Vec<[u8; 16]>,
        kwargs: Value,
    ) -> Result<Self, String> {
        let hex_addresses: Vec<String> = addresses.iter().map(|addr| hex::encode(addr)).collect();

        let command = json!({
            "command": "train",
            "tga_name": tga_name,
            "addresses": hex_addresses,
            "kwargs": kwargs
        });

        let result = Self::execute_python_command(&command)?;

        if let Some(error) = result.get("error") {
            return Err(format!("Python TGA training failed: {}", error));
        }

        let model_info = result
            .get("model_path")
            .and_then(|path| {
                Some(json!({
                    "model_path": path.as_str().unwrap(),
                    "tga_name": tga_name
                }))
            })
            .ok_or_else(|| "No model path in response".to_string())?;

        Ok(Self {
            tga_name: tga_name.to_string(),
            model_info: Some(model_info),
        })
    }

    pub fn generate_with_python(
        &self,
        count: usize,
        unique: bool,
        kwargs: Value,
    ) -> Result<Vec<[u8; 16]>, String> {
        let model_info = self
            .model_info
            .as_ref()
            .ok_or_else(|| "Model not trained".to_string())?;

        let command = json!({
            "command": "generate",
            "tga_name": &self.tga_name,
            "model_info": model_info,
            "count": count,
            "unique": unique,
            "kwargs": kwargs
        });

        let result = Self::execute_python_command(&command)?;

        if let Some(error) = result.get("error") {
            return Err(format!("Python TGA generation failed: {}", error));
        }

        let addresses = result
            .get("addresses")
            .and_then(|addrs| addrs.as_array())
            .ok_or_else(|| "No addresses in response".to_string())?;

        let mut result_addresses = Vec::new();
        for addr_hex in addresses {
            let hex_str = addr_hex
                .as_str()
                .ok_or_else(|| "Invalid address format".to_string())?;

            let bytes =
                hex::decode(hex_str).map_err(|e| format!("Failed to decode hex address: {}", e))?;

            if bytes.len() != 16 {
                return Err(format!("Invalid address length: {}", bytes.len()));
            }

            let mut addr = [0u8; 16];
            addr.copy_from_slice(&bytes);
            result_addresses.push(addr);
        }

        Ok(result_addresses)
    }

    fn execute_python_command(command: &Value) -> Result<Value, String> {
        let script_path = Self::find_python_script()?;

        let python_executable = Self::find_python_executable()?;

        println!("[DEBUG] Using Python executable: {}", python_executable);
        println!("[DEBUG] Using script path: {:?}", script_path);

        let mut child = Command::new(&python_executable)
            .arg(&script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start Python subprocess: {}", e))?;

        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "Failed to get stdin".to_string())?;

        let command_str = serde_json::to_string(command)
            .map_err(|e| format!("Failed to serialize command: {}", e))?;

        println!("[DEBUG] Sending command: {}", command_str);

        writeln!(stdin, "{}", command_str)
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to get stdout".to_string())?;

        let reader = BufReader::new(stdout);
        let mut response = String::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read stdout: {}", e))?;
            response = line;
            break;
        }

        println!("[DEBUG] Received response: '{}'", response);

        let status = child
            .wait()
            .map_err(|e| format!("Failed to wait for subprocess: {}", e))?;

        if !status.success() {
            let stderr = child
                .stderr
                .map(|mut stderr| {
                    let mut error_output = String::new();
                    std::io::Read::read_to_string(&mut stderr, &mut error_output)
                        .map(|_| error_output)
                        .unwrap_or_else(|_| "Unknown error".to_string())
                })
                .unwrap_or_else(|| "No stderr available".to_string());

            return Err(format!("Python subprocess failed: {}", stderr));
        }

        serde_json::from_str(&response).map_err(|e| format!("Failed to parse JSON response: {}", e))
    }

    fn find_python_script() -> Result<PathBuf, String> {
        let mut script_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get current executable path: {}", e))?
            .parent()
            .ok_or_else(|| "Failed to get executable directory".to_string())?
            .to_path_buf();

        script_path.push("python");
        script_path.push("tga_runner.py");

        if script_path.exists() {
            return Ok(script_path);
        }

        let mut script_path = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?
            .join("python")
            .join("tga_runner.py");

        if script_path.exists() {
            return Ok(script_path);
        }

        let mut script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        script_path.push("python");
        script_path.push("tga_runner.py");

        if script_path.exists() {
            return Ok(script_path);
        }

        Err("Could not find tga_runner.py script".to_string())
    }

    fn find_python_executable() -> Result<String, String> {
        let mut venv_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        venv_path.push("python");
        venv_path.push("venv");
        venv_path.push("bin");
        venv_path.push("python");

        if venv_path.exists() {
            return Ok(venv_path.to_string_lossy().to_string());
        }

        Ok("python3".to_string())
    }
}

#[typetag::serde]
impl TGA for PythonTGA {
    fn train<T: IntoIterator<Item = [u8; 16]>>(seeds: T) -> Result<Self, String>
    where
        Self: Sized,
    {
        let addresses: Vec<[u8; 16]> = seeds.into_iter().collect();
        let kwargs = serde_json::json!({});
        Self::train_with_python("lstm_ipv6", addresses, kwargs)
    }

    fn generate(&self) -> [u8; 16] {
        let kwargs = serde_json::json!({});
        let addresses = self
            .generate_with_python(1, false, kwargs)
            .expect("Failed to generate address");
        addresses[0]
    }

    fn name(&self) -> &'static str {
        "python_tga"
    }

    fn description(&self) -> &'static str {
        "Python-based TGA using subprocess communication"
    }
}


static PYTHON_TGAS_INIT: Once = Once::new();
static PYTHON_TGAS: Mutex<Vec<PythonTgaInfo>> = Mutex::new(Vec::new());

pub fn get_available_python_tga_infos() -> Result<Vec<PythonTgaInfo>, String> {
    PYTHON_TGAS_INIT.call_once(|| match query_python_tgas() {
        Ok(tgas) => {
            let mut python_tgas = PYTHON_TGAS.lock().unwrap();
            *python_tgas = tgas;
        }
        Err(e) => {
            eprintln!("Warning: Failed to query Python TGAs: {}", e);
        }
    });

    Ok(PYTHON_TGAS.lock().unwrap().clone())
}

fn query_python_tgas() -> Result<Vec<PythonTgaInfo>, String> {
    let command = json!({
        "command": "list_tgas"
    });

    let result = PythonTGA::execute_python_command(&command)?;

    if let Some(error) = result.get("error") {
        return Err(format!("Failed to query Python TGAs: {}", error));
    }

    let tgas = result
        .get("tgas")
        .and_then(|tgas| tgas.as_array())
        .ok_or_else(|| "No TGAs in response".to_string())?;

    let mut python_tgas = Vec::new();
    for tga in tgas {
        let name = tga
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| "Missing TGA name".to_string())?;

        let description = tga
            .get("description")
            .and_then(|d| d.as_str())
            .ok_or_else(|| "Missing TGA description".to_string())?;

        python_tgas.push(PythonTgaInfo {
            name: name.to_string(),
            description: description.to_string(),
        });
    }

    Ok(python_tgas)
}

pub fn get_build_time_python_tgas() -> Vec<(String, String)> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_tga_discovery() {
        let result = get_available_python_tga_infos();
        match result {
            Ok(tgas) => {
                println!("Found {} Python TGAs:", tgas.len());
                for tga in tgas {
                    println!("  {}: {}", tga.name, tga.description);
                }
            }
            Err(e) => {
                println!(
                    "Python TGA discovery failed (expected if Python not available): {}",
                    e
                );
            }
        }
    }
}
