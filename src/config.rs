use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct RootConfig {
    #[serde(default)]
    pub lt_enabled: bool, 
    #[serde(default)]
    pub lt_api_hostname: String, 
    #[serde(default)]
    pub lt_api_port: String, 

    #[serde(default)]
    pub completion_enabled: bool, 
}

impl Default for RootConfig {
    fn default() -> Self {
        Self { 
            lt_enabled: true,
            lt_api_hostname: "http://127.0.0.1".to_string(),
            lt_api_port: "8081".to_string(),

            completion_enabled: true,

        }
    }
}
impl RootConfig {
    pub fn init_from_file(json_file_path: &Path) -> Option<Self> {
        let file_str :String;
        if let Ok(filet) = fs::read_to_string(json_file_path) {
            println!("Reading config from {}", json_file_path.display());
            file_str = filet;
        } else if let Ok(filet) = fs::read_to_string("config.json") {
            println!("Reading config from {}", "config.json");
            file_str = filet;
        } else {
            println!("Did not find a config file");
            return None;
        }
        let u: RootConfig = match serde_json::from_str(&file_str) {
            Ok(c) => c,
            Err(e) => {
                println!("Parse error: {}", e);
                return None;
            }
        };
        Some(u)
    }
}

