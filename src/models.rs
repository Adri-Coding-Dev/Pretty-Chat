//! Modelos de datos comunes a todos los módulos.

/// Representa un mensaje de chat normalizado.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub username: String,
    pub content: String,
    pub timestamp: u64,          // epoch en milisegundos
    pub badges: Vec<Badge>,
    pub color: Option<Color>,
    pub is_superchat: bool,
    pub superchat_amount: Option<String>,
}

/// Insignias que puede tener un usuario.
#[derive(Debug, Clone, PartialEq)]
pub enum Badge {
    Moderator,
    Verified,
    Member,
    Custom(String),
}

/// Color RGB simple.
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Información pública de la transmisión en vivo.
#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub title: String,
    pub channel_name: String,
    pub viewers: String,        // espectadores concurrentes
    pub like_count: String,     // total de "me gusta"
    pub view_count: String,     // reproducciones totales
    pub is_live: bool,
}
