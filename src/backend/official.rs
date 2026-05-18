//! Backend oficial que utiliza la API de YouTube Data v3.
//! Requiere una clave de API válida almacenada en la variable de entorno YOUTUBE_API_KEY.

use async_trait::async_trait;
use reqwest::Client;
use tokio::sync::mpsc;
use tracing::{info, warn, error};
use crate::models::{Badge, ChatMessage};
use super::ChatBackend;

pub struct OfficialBackend {
    video_id: String,
    api_key: String,
    client: Client,
    live_chat_id: Option<String>,
}

impl OfficialBackend {
    /// Crea una nueva instancia del backend oficial.
    /// `video_url` es la URL del directo de YouTube.
    /// `api_key` es la clave de API de YouTube Data v3.
    pub fn new(video_url: String, api_key: String) -> Self {
        let video_id = extract_video_id(&video_url).unwrap_or_default();
        Self {
            video_id,
            api_key,
            client: Client::new(),
            live_chat_id: None,
        }
    }

    /// Obtiene el `liveChatId` necesario para leer los mensajes.
    async fn resolve_live_chat_id(&mut self) -> color_eyre::Result<String> {
        let url = format!(
            "https://www.googleapis.com/youtube/v3/videos?part=liveStreamingDetails&id={}&key={}",
            self.video_id, self.api_key
        );
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(color_eyre::eyre::eyre!("Error al obtener detalles del vídeo: {}", resp.status()));
        }
        let json: serde_json::Value = resp.json().await?;
        let chat_id = json
            .pointer("/items/0/liveStreamingDetails/activeLiveChatId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| color_eyre::eyre::eyre!("No se encontró activeLiveChatId. ¿Está el chat en vivo activo?"))?
            .to_string();

        info!("liveChatId obtenido: {}", chat_id);
        self.live_chat_id = Some(chat_id.clone());
        Ok(chat_id)
    }

    /// Realiza una petición a la API para obtener mensajes.
    async fn poll_chat(
        &self,
        live_chat_id: &str,
        page_token: Option<&str>,
        tx: &mpsc::UnboundedSender<ChatMessage>,
    ) -> color_eyre::Result<(Option<String>, u64)> {
        let mut url = format!(
            "https://www.googleapis.com/youtube/v3/liveChat/messages?liveChatId={}&part=snippet,authorDetails&key={}",
            live_chat_id, self.api_key
        );
        if let Some(token) = page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            warn!("HTTP error: {}", resp.status());
            return Ok((None, 5000));
        }

        let json: serde_json::Value = resp.json().await?;
        let items = json["items"].as_array();

        if let Some(items) = items {
            for item in items {
                let msg = Self::parse_message(item);
                if tx.send(msg).is_err() {
                    return Ok((None, 5000)); // Canal cerrado
                }
            }
        }

        let next_page_token = json["nextPageToken"].as_str().map(|s| s.to_string());
        let interval = json["pollingIntervalMillis"].as_u64().unwrap_or(5000);

        Ok((next_page_token, interval))
    }

    /// Convierte un item JSON de la API en un `ChatMessage`.
    fn parse_message(item: &serde_json::Value) -> ChatMessage {
        let snippet = &item["snippet"];
        let author = &item["authorDetails"];

        let username = author["displayName"].as_str().unwrap_or("unknown").to_string();
        let content = snippet["displayMessage"].as_str().unwrap_or("").to_string();

        let is_superchat = snippet["type"].as_str() == Some("superChatEvent");
        let superchat_amount = if is_superchat {
            snippet["superChatDetails"]["amountDisplayString"].as_str().map(|s| s.to_string())
        } else {
            None
        };

        let timestamp = snippet["publishedAt"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp_millis() as u64)
            .unwrap_or(0);

        let is_moderator = author["isChatModerator"].as_bool().unwrap_or(false);
        let is_verified = author["isVerified"].as_bool().unwrap_or(false);
        let is_sponsor = author["isChatSponsor"].as_bool().unwrap_or(false);

        let mut badges = Vec::new();
        if is_moderator { badges.push(Badge::Moderator); }
        if is_verified { badges.push(Badge::Verified); }
        if is_sponsor { badges.push(Badge::Member); }

        ChatMessage {
            username,
            content,
            timestamp,
            badges,
            color: None,
            is_superchat,
            superchat_amount,
        }
    }
}

#[async_trait]
impl ChatBackend for OfficialBackend {
    async fn run(mut self: Box<Self>, tx: mpsc::UnboundedSender<ChatMessage>) {
        let live_chat_id = match self.resolve_live_chat_id().await {
            Ok(id) => id,
            Err(e) => {
                error!("Error al obtener el ID del chat en vivo: {}", e);
                return;
            }
        };

        let mut page_token: Option<String> = None;
        loop {
            match self.poll_chat(&live_chat_id, page_token.as_deref(), &tx).await {
                Ok((token, interval)) => {
                    page_token = token;
                    tokio::time::sleep(std::time::Duration::from_millis(interval)).await;
                }
                Err(e) => {
                    error!("Error en poll_chat: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
}

/// Extrae el ID del vídeo de una URL de YouTube.
fn extract_video_id(url: &str) -> Option<String> {
    if let Some(pos) = url.find("v=") {
        let start = pos + 2;
        let end = url[start..].find('&').map(|e| start + e).unwrap_or(url.len());
        Some(url[start..end].to_string())
    } else if url.contains("youtu.be/") {
        let start = url.rfind('/').map(|p| p + 1).unwrap_or(0);
        let end = url[start..].find('?').map(|e| start + e).unwrap_or(url.len());
        Some(url[start..end].to_string())
    } else {
        None
    }
}
