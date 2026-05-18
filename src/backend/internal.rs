//! Backend interno (scraping) que intenta leer el chat desde las variables JavaScript de la página.
//! Actualmente no funcional debido a cambios en YouTube, se mantiene como referencia.

use async_trait::async_trait;
use reqwest::Client;
use tokio::sync::mpsc;
use tracing::{info, warn, error};
use crate::models::{Badge, ChatMessage, Color};
use super::ChatBackend;
use fastrand::Rng;

pub struct InternalBackend {
    video_url: String,
    client: Client,
    api_key: Option<String>,
    context: serde_json::Value,
}

impl InternalBackend {
    pub fn new(video_url: String) -> Self {
        let client = Client::builder()
            .cookie_store(true)
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .expect("Failed to build HTTP client");

        let context = serde_json::json!({
            "client": {
                "clientName": "WEB",
                "clientVersion": "2.20250416.00.00",
                "hl": "en",
                "gl": "US",
                "utcOffsetMinutes": 0,
            }
        });

        Self {
            video_url,
            client,
            api_key: None,
            context,
        }
    }

    async fn fetch_initial_data(&mut self) -> color_eyre::Result<(String, String)> {
        let resp = self.client.get(&self.video_url).send().await?;
        if !resp.status().is_success() {
            return Err(color_eyre::eyre::eyre!("HTTP status: {}", resp.status()));
        }
        let html = resp.text().await?;

        // Extraer clave de API
        let api_key = html.split("\"innertubeApiKey\":\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .map(|s| s.to_string())
            .ok_or_else(|| color_eyre::eyre::eyre!("No se encontró innertubeApiKey"))?;
        self.api_key = Some(api_key.clone());

        // Intentar obtener el primer continuation token
        if let Some(token) = try_extract_json_from_script("var ytInitialPlayerResponse = ", &html)
            .and_then(|json| find_continuation_recursive(&json))
        {
            info!("Continuation token encontrado en ytInitialPlayerResponse");
            return Ok((api_key, token));
        }
        if let Some(token) = try_extract_json_from_script("var ytInitialData = ", &html)
            .and_then(|json| find_continuation_recursive(&json))
        {
            info!("Continuation token encontrado en ytInitialData");
            return Ok((api_key, token));
        }
        if let Some(token) = find_continuation_in_html(&html) {
            info!("Continuation token encontrado en HTML con liveChatRenderer");
            return Ok((api_key, token));
        }

        let snippet: String = html.chars().take(3000).collect();
        error!("No se encontró continuation token en el HTML. Primeros 3000 caracteres:\n{}", snippet);
        Err(color_eyre::eyre::eyre!(
            "No se encontró continuation token. Esto puede deberse a que el chat en vivo no está activo o a que YouTube ha cambiado la carga de la página.\n\
             Puedes probar la interfaz con: pchat --backend mock <url>"
        ))
    }

    async fn poll_chat(
        &self,
        api_key: &str,
        continuation: &str,
        tx: &mpsc::UnboundedSender<ChatMessage>,
    ) -> color_eyre::Result<Option<String>> {
        let url = format!(
            "https://www.youtube.com/youtubei/v1/live_chat/get_live_chat?key={}",
            api_key
        );

        let payload = serde_json::json!({
            "context": self.context,
            "continuation": continuation,
        });

        let resp = self.client.post(&url).json(&payload).send().await?;
        if !resp.status().is_success() {
            warn!("HTTP error: {}", resp.status());
            return Ok(None);
        }

        let json: serde_json::Value = resp.json().await?;
        let actions = json
            .pointer("/continuationContents/liveChatContinuation/actions")
            .and_then(|v| v.as_array());

        if let Some(actions) = actions {
            for action in actions {
                if let Some(item) = action
                    .pointer("/addChatItemAction/item/liveChatTextMessageRenderer")
                    .or_else(|| action.pointer("/addChatItemAction/item/liveChatPaidMessageRenderer"))
                {
                    let msg = Self::parse_message(item);
                    if tx.send(msg).is_err() {
                        return Ok(None);
                    }
                }
            }
        }

        let next_continuation = find_continuation_recursive(&json);
        Ok(next_continuation)
    }

    fn parse_message(item: &serde_json::Value) -> ChatMessage {
        let username = item
            .pointer("/authorName/simpleText")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let text_runs = item.pointer("/message/runs");
        let content = if let Some(runs) = text_runs.and_then(|v| v.as_array()) {
            runs.iter()
                .filter_map(|r| r.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<&str>>()
                .join("")
        } else {
            item.pointer("/message/simpleText")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        };

        let mut badges = Vec::new();
        if let Some(badges_json) = item.pointer("/authorBadges").and_then(|v| v.as_array()) {
            for badge in badges_json {
                if let Some(icon_type) = badge
                    .pointer("/liveChatAuthorBadgeRenderer/icon/iconType")
                    .and_then(|v| v.as_str())
                {
                    match icon_type {
                        "MODERATOR" => badges.push(Badge::Moderator),
                        "VERIFIED" => badges.push(Badge::Verified),
                        "OWNER" | "MEMBERSHIP" => badges.push(Badge::Member),
                        _ => {}
                    }
                }
            }
        }

        let color = item
            .pointer("/authorName/simpleTextColor")
            .and_then(|v| v.as_u64())
            .map(|num| {
                let rgb = num as u32;
                Color {
                    r: ((rgb >> 16) & 0xFF) as u8,
                    g: ((rgb >> 8) & 0xFF) as u8,
                    b: (rgb & 0xFF) as u8,
                }
            });

        let timestamp_usec = item
            .pointer("/timestampUsec")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let timestamp = timestamp_usec / 1000;

        let is_superchat = item.pointer("/purchaseAmountText/simpleText").is_some();
        let superchat_amount = item
            .pointer("/purchaseAmountText/simpleText")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        ChatMessage {
            username,
            content,
            timestamp,
            badges,
            color,
            is_superchat,
            superchat_amount,
        }
    }
}

#[async_trait]
impl ChatBackend for InternalBackend {
    async fn run(mut self: Box<Self>, tx: mpsc::UnboundedSender<ChatMessage>) {
        info!("Conectando al chat en vivo...");

        let (api_key, mut continuation) = match self.fetch_initial_data().await {
            Ok(data) => data,
            Err(e) => {
                error!("Error al obtener datos iniciales: {}", e);
                return;
            }
        };

        let mut rng = Rng::new();
        loop {
            match self.poll_chat(&api_key, &continuation, &tx).await {
                Ok(Some(next)) => continuation = next,
                Ok(None) => {
                    info!("Chat finalizado o sin nuevos mensajes, reintentando...");
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                }
                Err(e) => {
                    error!("Error en poll_chat: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }

            let delay = 500 + rng.u32(0..500);
            tokio::time::sleep(std::time::Duration::from_millis(delay as u64)).await;
        }
    }
}

// --------------------------------------------------
// Utilidades de scraping (ya no funcionales)
// --------------------------------------------------

fn try_extract_json_from_script(prefix: &str, html: &str) -> Option<serde_json::Value> {
    let start = html.find(prefix)? + prefix.len();
    let slice = &html[start..];
    let json_str = slice.split(";</script>").next()?;
    let json_str = json_str.trim_end_matches(';');
    serde_json::from_str(json_str).ok()
}

fn find_continuation_in_html(html: &str) -> Option<String> {
    let mut search_start = 0;
    while let Some(pos) = html[search_start..].find("\"liveChatRenderer\":") {
        let abs_pos = search_start + pos;
        let slice = &html[abs_pos + "\"liveChatRenderer\":".len()..];
        if let Some(json_str) = extract_json_object(slice) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                if let Some(token) = find_continuation_recursive(&json) {
                    return Some(token);
                }
            }
        }
        search_start = abs_pos + 1;
    }
    None
}

fn extract_json_object(s: &str) -> Option<String> {
    let start_idx = s.find('{')?;
    let chars: Vec<char> = s[start_idx..].chars().collect();
    let mut count = 0;
    let mut end_idx = 0;
    for (i, ch) in chars.iter().enumerate() {
        if *ch == '{' {
            count += 1;
        } else if *ch == '}' {
            count -= 1;
            if count == 0 {
                end_idx = i + 1;
                break;
            }
        }
    }
    if end_idx > 0 {
        Some(chars[..end_idx].iter().collect())
    } else {
        None
    }
}

fn find_continuation_recursive(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                if key.ends_with("ContinuationData") {
                    if let Some(cont) = val.get("continuation").and_then(|v| v.as_str()) {
                        return Some(cont.to_string());
                    }
                }
            }
            for val in map.values() {
                if let result @ Some(_) = find_continuation_recursive(val) {
                    return result;
                }
            }
            None
        }
        serde_json::Value::Array(arr) => {
            for val in arr {
                if let result @ Some(_) = find_continuation_recursive(val) {
                    return result;
                }
            }
            None
        }
        _ => None,
    }
}
