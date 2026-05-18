//! Capa de abstracción para fuentes de mensajes de chat.
//! Define el trait `ChatBackend` que deben implementar todos los backends.

use async_trait::async_trait;
use tokio::sync::mpsc;
use crate::models::ChatMessage;

/// Trait común para cualquier backend de chat.
#[async_trait]
pub trait ChatBackend: Send + Sync {
    /// Inicia la obtención de mensajes.
    /// Los mensajes se envían a través del canal `tx`.
    /// El método debe finalizar si el receptor se cierra o si ocurre un error irrecuperable.
    async fn run(self: Box<Self>, tx: mpsc::UnboundedSender<ChatMessage>);
}

pub mod internal;
pub mod official;
pub mod mock;
