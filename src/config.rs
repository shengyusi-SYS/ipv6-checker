use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    pub urls: Vec<String>,
}

fn default_port() -> u16 {
    3443
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn default() -> Self {
        Config {
            port: default_port(),
            urls: vec![
                "https://6.ipw.cn".to_string(),
                "http://checkipv6.dyndns.com/".to_string(),
            ],
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();

        match Self::load(path) {
            Ok(config) => {
                tracing::info!("Loaded config from {:?}", path);
                config
            }
            Err(e) => {
                tracing::warn!("Failed to load config: {}. Creating default config", e);
                let config = Self::default();

                // 尝试创建默认配置文件
                match config.save(path) {
                    Ok(_) => {
                        tracing::info!("Created default config file at {:?}", path);
                    }
                    Err(e) => {
                        tracing::error!("Failed to create config file: {}", e);
                    }
                }

                config
            }
        }
    }
}
