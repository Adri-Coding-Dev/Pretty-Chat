//! Backend de demostración que genera mensajes simulados.

use async_trait::async_trait;
use tokio::sync::mpsc;
use crate::models::{Badge, ChatMessage, Color};
use super::ChatBackend;

pub struct MockBackend;

#[async_trait]
impl ChatBackend for MockBackend {
    async fn run(self: Box<Self>, tx: mpsc::UnboundedSender<ChatMessage>) {
        let usernames = [
            "alice", "bob", "mod_carol", "vip_dave", "eve", "fan42",
            "chat_guru", "night_owl", "tech_nerd", "pixel_queen",
        ];
        let messages = [
            "Hola a todos!",
            "Que tal el directo?",
            "Esto es increible",
            "Mensaje largo de prueba: Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
            "Rust mola",
            "Me encanta la terminal",
            "LIVE desde mi terminal",
            "Alguien mas esta viendo esto?",
            "Que pasada!",
            "Suscribanse para mas contenido",
        ];
        let badges_pool = [
            vec![],
            vec![Badge::Moderator],
            vec![Badge::Verified],
            vec![Badge::Member],
            vec![Badge::Moderator, Badge::Verified],
        ];

        let mut counter = 0u64;
        loop {
            counter += 1;
            let user_idx = counter as usize % usernames.len();
            let msg_idx = counter as usize % messages.len();
            let badge_idx = counter as usize % badges_pool.len();

            let msg = ChatMessage {
                username: usernames[user_idx].to_string(),
                content: messages[msg_idx].to_string(),
                timestamp: counter * 1000,
                badges: badges_pool[badge_idx].clone(),
                color: Some(Color {
                    r: (50 + (counter * 30) % 200) as u8,
                    g: (100 + (counter * 50) % 150) as u8,
                    b: (150 + (counter * 40) % 100) as u8,
                }),
                is_superchat: counter % 15 == 0,
                superchat_amount: if counter % 15 == 0 {
                    Some("$5.00".to_string())
                } else {
                    None
                },
            };

            if tx.send(msg).is_err() {
                break; // La aplicación se cerró
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }
}
