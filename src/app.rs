//! Gestión del estado de la aplicación: mensajes, estadísticas y configuración visual.

use tokio::sync::mpsc;
use crate::models::ChatMessage;
use crate::themes::Theme;

/// Estructura principal que mantiene el estado de la aplicación.
pub struct App {
    /// Lista de mensajes recibidos.
    pub messages: Vec<ChatMessage>,
    /// Contador total de mensajes desde el inicio.
    pub total_messages: u64,
    /// Receptor asíncrono de nuevos mensajes.
    pub rx: mpsc::UnboundedReceiver<ChatMessage>,
    /// Indica si se usa el modo compacto (menos información).
    pub compact: bool,
    /// Tema cargado (None = usar colores por defecto).
    pub theme: Option<Theme>,
    /// Información del stream en vivo (si está disponible).
    pub stream_info: Option<StreamInfo>,

    // Ventana deslizante de timestamps para calcular mensajes por segundo.
    recent_timestamps: Vec<u64>,
    // Contador de mensajes marcados como spam (futuro uso).
    spam_counter: u64,
    // Longitud actual de la cola de mensajes (para estadísticas).
    queue_len: usize,
}

/// Información pública del stream.
pub struct StreamInfo {
    pub title: String,
    pub channel_name: String,
    pub viewers: String,
    pub like_count: String,
    pub view_count: String,
    pub is_live: bool,
}

impl App {
    /// Crea una nueva instancia de la aplicación.
    pub fn new(rx: mpsc::UnboundedReceiver<ChatMessage>, compact: bool, theme: Option<Theme>) -> Self {
        Self {
            messages: Vec::with_capacity(2000),
            total_messages: 0,
            rx,
            compact,
            theme,
            stream_info: None,
            recent_timestamps: Vec::with_capacity(100),
            spam_counter: 0,
            queue_len: 0,
        }
    }

    /// Intenta recibir todos los mensajes pendientes sin bloquear.
    pub fn update(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            self.total_messages += 1;
            self.recent_timestamps.push(msg.timestamp);
            if self.recent_timestamps.len() > 100 {
                self.recent_timestamps.remove(0);
            }
            self.messages.push(msg);
        }
        self.queue_len = self.rx.len();
    }

    /// Devuelve los últimos mensajes que caben en la altura indicada.
    pub fn visible_messages(&self, height: usize) -> &[ChatMessage] {
        let start = if self.messages.len() > height {
            self.messages.len() - height
        } else {
            0
        };
        &self.messages[start..]
    }

    /// Calcula la tasa de mensajes por segundo usando una ventana de 100 mensajes.
    pub fn messages_per_second(&self) -> f64 {
        if self.recent_timestamps.len() < 2 {
            return 0.0;
        }
        let first = self.recent_timestamps.first().unwrap();
        let last = self.recent_timestamps.last().unwrap();
        let dt = (last - first) as f64 / 1000.0;
        if dt <= 0.0 {
            return self.recent_timestamps.len() as f64;
        }
        self.recent_timestamps.len() as f64 / dt
    }

    /// Proporción de mensajes considerados spam (actualmente 0, reservado para filtros).
    pub fn spam_ratio(&self) -> f64 {
        if self.total_messages == 0 {
            0.0
        } else {
            self.spam_counter as f64 / self.total_messages as f64
        }
    }

    /// Número de mensajes en la cola aún sin procesar.
    pub fn queue_len(&self) -> usize {
        self.queue_len
    }
}
