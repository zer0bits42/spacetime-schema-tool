use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;

pub struct SpacetimeClient {
    client: Client,
    base_url: String,
}

impl SpacetimeClient {
    pub fn new(server: &str) -> Result<Self> {
        let base_url = get_server_url(server)?;

        Ok(Self {
            client: Client::new(),
            base_url,
        })
    }

    pub async fn fetch_schema(&self, database: &str, version: Option<String>) -> Result<Value> {
        let version = version.unwrap_or_else(|| "9".to_string());
        let url = format!(
            "{}/v1/database/{}/schema?version={}",
            self.base_url, database, version
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Schema fetch failed: {}", error_text));
        }

        let schema_text = response.text().await?;
        Ok(serde_json::from_str(&schema_text)?)
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Get server URL for a nickname (e.g., "local" -> <http://127.0.0.1:3000>)
fn get_server_url(server: &str) -> Result<String> {
    // Handle full URLs
    if server.starts_with("http://") || server.starts_with("https://") {
        return Ok(server.to_string());
    }

    // Check SpacetimeDB CLI config for server nicknames
    let cli_config_path = get_spacetime_cli_config_path()?;
    if cli_config_path.exists() {
        let content = std::fs::read_to_string(&cli_config_path)?;
        let config: toml::Value = toml::from_str(&content)?;

        if let Some(server_configs) = config.get("server_configs").and_then(|v| v.as_array()) {
            for server_config in server_configs {
                if let Some(nickname) = server_config.get("nickname").and_then(|v| v.as_str()) {
                    if nickname == server {
                        if let (Some(protocol), Some(host)) = (
                            server_config.get("protocol").and_then(|v| v.as_str()),
                            server_config.get("host").and_then(|v| v.as_str()),
                        ) {
                            return Ok(format!("{protocol}://{host}"));
                        }
                    }
                }
            }
        }
    }

    // Default fallback
    match server {
        "local" => Ok("http://localhost:3000".to_string()),
        "cloud" | "maincloud" => Ok("https://maincloud.spacetimedb.com".to_string()),
        _ => Ok(format!("http://{server}")),
    }
}

fn get_spacetime_cli_config_path() -> Result<std::path::PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    Ok(home.join(".config").join("spacetime").join("cli.toml"))
}