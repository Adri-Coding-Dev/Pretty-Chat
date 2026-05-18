//! Carga de la configuración del usuario desde ~/.config/pchat/config.toml

use serde::Deserialize;
use std::fs;

/// Configuración de la aplicación.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub theme: Option<String>,
    pub default_backend: Option<String>,
    pub compact: Option<bool>,
}

impl AppConfig {
    /// Carga el archivo de configuración o devuelve un valor por defecto.
    pub fn load() -> color_eyre::Result<Self> {
        let config_path = dirs::config_dir()
            .unwrap()
            .join("pchat/config.toml");
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(AppConfig::default())
        }
    }
}
