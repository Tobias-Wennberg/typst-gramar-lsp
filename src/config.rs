use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct RootConfig {
    #[serde(default)]
    pub lt_enabled: bool, 
    #[serde(default)]
    pub lt_api_hostname: String, 
    #[serde(default)]
    pub lt_api_port: String, 
}

impl Default for RootConfig {
    fn default() -> Self {
        Self { 
            lt_enabled: true,
            lt_api_hostname: "http://127.0.0.1".to_string(),
            lt_api_port: "8081".to_string(),

        }
    }
}
impl RootConfig {
    pub fn init_from_file(json_file_path: &Path) -> Option<Self> {
        let file = match File::open(json_file_path) {
            Ok(f) => f,
            Err(e) => {
                println!("Open file error: {}", e);
                return None;
            }
        };
        let reader = BufReader::new(file);
        let u: RootConfig = match serde_json::from_reader(reader) {
            Ok(c) => c,
            Err(e) => {
                println!("Parse error: {}", e);
                return None;
            }
        };
        Some(u)
    }
}

