//! Carga de temas desde archivos TOML.

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// Estructura del archivo de tema.
#[derive(Debug, Deserialize, Clone)]
pub struct Theme {
    pub colors: ThemeColors,
}

/// Paleta de colores del tema.
#[derive(Debug, Deserialize, Clone)]
pub struct ThemeColors {
    pub background: String,
    pub foreground: String,
    pub border: String,
    pub moderator: String,
    pub verified: String,
    pub member: String,
    pub superchat: String,
    pub info: String,
    pub alert: String,
}

impl Theme {
    /// Carga un tema por nombre (sin extensión).
    /// Busca en ~/.config/pchat/themes/ y luego en ./themes/ del proyecto.
    pub fn load(name: &str) -> color_eyre::Result<Self> {
        let themes_dir = dirs::config_dir()
            .unwrap()
            .join("pchat/themes");
        let path = themes_dir.join(format!("{name}.toml"));
        if !path.exists() {
            // Fallback a directorio local
            let local = PathBuf::from("themes").join(format!("{name}.toml"));
            let content = fs::read_to_string(&local)?;
            return Ok(toml::from_str(&content)?);
        }
        let content = fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    }
}

/// Convierte un string hexadecimal (#RRGGBB) a tupla (R, G, B).
pub fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}
