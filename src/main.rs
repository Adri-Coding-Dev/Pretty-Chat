//! Punto de entrada de la aplicación pchat.
//! Configura la terminal, parsea los argumentos de línea de comandos,
//! selecciona el backend adecuado y ejecuta el bucle principal de la TUI.

mod app;
mod backend;
mod config;
mod models;
mod renderer;
mod themes;

use app::App;
use backend::{ChatBackend, internal::InternalBackend, official::OfficialBackend, mock::MockBackend};
use clap::Parser;
use config::AppConfig;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::sync::mpsc;
use tracing::info;
use models::StreamInfo;

/// Estructura que define los argumentos de línea de comandos.
#[derive(Parser)]
#[command(name = "pchat")]
struct Cli {
    /// URL del vídeo de YouTube (live)
    url: String,

    /// Backend a usar: internal, official, mock
    #[arg(long, default_value = "internal")]
    backend: String,

    /// Nombre del tema (ej. tokyo-night-enhanced, nord, etc.)
    #[arg(long)]
    theme: Option<String>,

    /// Activa el modo compacto (menos información)
    #[arg(long)]
    compact: bool,

    /// Activa el modo expandido (por defecto)
    #[arg(long)]
    expanded: bool,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // Inicializar el sistema de logging (tracing)
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Cargar configuración global desde ~/.config/pchat/config.toml
    let config = AppConfig::load().unwrap_or_default();

    // Resolver tema: prioridad CLI > archivo config > "tokyo-night-enhanced"
    let theme_name = cli.theme.or(config.theme).unwrap_or_else(|| "tokyo-night-enhanced".into());
    let theme = match themes::Theme::load(&theme_name) {
        Ok(t) => Some(t),
        Err(e) => {
            tracing::warn!("No se pudo cargar el tema {}: {}", theme_name, e);
            None
        }
    };

    // Determinar modo compacto
    let compact = if cli.compact {
        true
    } else if cli.expanded {
        false
    } else {
        config.compact.unwrap_or(false)
    };

    // Configuración de la terminal: modo raw y pantalla alternativa
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Canal de mensajes entre el backend y la UI
    let (tx, rx) = mpsc::unbounded_channel();

    // Selección del backend de chat
    let chat_backend: Box<dyn ChatBackend> = match cli.backend.as_str() {
        "internal" => Box::new(InternalBackend::new(cli.url)),
        "official" => {
            let api_key = std::env::var("YOUTUBE_API_KEY")
                .unwrap_or_else(|_| {
                    eprintln!("Error: La variable de entorno YOUTUBE_API_KEY no está definida.");
                    std::process::exit(1);
                });
            Box::new(OfficialBackend::new(cli.url, api_key))
        },
        "mock" => {
            info!("Usando backend de demostración (mock)");
            Box::new(MockBackend)
        },
        other => {
            eprintln!("Backend desconocido: {}", other);
            return Ok(());
        }
    };

    // Lanzar el backend en una tarea asíncrona separada
    tokio::spawn(async move {
        chat_backend.run(tx).await;
    });

    // Obtener información del stream (título, canal, etc.)
    let stream_info = match cli.backend.as_str() {
        "official" => {
            let api_key = std::env::var("YOUTUBE_API_KEY")
                .expect("YOUTUBE_API_KEY no definida");
            let video_id = extract_video_id(&cli.url).unwrap_or_default();
            fetch_stream_info(&video_id, &api_key).await.ok()
        },
        "mock" => Some(StreamInfo {
            title: "pchat live demo".to_string(),
            channel_name: "Rust TUI Channel".to_string(),
            viewers: "1.2K".to_string(),
            like_count: "3.4K".to_string(),
            view_count: "45K".to_string(),
            is_live: true,
        }),
        _ => None,
    };

    // Crear la aplicación y ejecutar el bucle principal
    let mut app = App::new(rx, compact, theme);
    app.stream_info = stream_info;

    let res = run_app(&mut app, terminal).await;

    // Restaurar la terminal antes de salir
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

    res
}

/// Bucle principal de la interfaz de usuario.
async fn run_app(
    app: &mut App,
    mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
) -> color_eyre::Result<()> {
    loop {
        // Procesar nuevos mensajes entrantes
        app.update();

        // Renderizar la interfaz
        terminal.draw(|f| renderer::draw(f, app))?;

        // Esperar eventos de teclado con un timeout corto para mantener fluidez
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') => app.compact = !app.compact,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

/// Extrae el identificador de vídeo de una URL de YouTube.
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

/// Obtiene información del stream desde la API oficial de YouTube.
async fn fetch_stream_info(video_id: &str, api_key: &str) -> color_eyre::Result<StreamInfo> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/videos?part=snippet,statistics,liveStreamingDetails&id={}&key={}",
        video_id, api_key
    );
    let resp = reqwest::get(&url).await?;
    if !resp.status().is_success() {
        return Err(color_eyre::eyre::eyre!("Error al obtener información del vídeo: {}", resp.status()));
    }
    let json: serde_json::Value = resp.json().await?;
    let item = &json["items"][0];

    let title = item["snippet"]["title"].as_str().unwrap_or("").to_string();
    let channel_name = item["snippet"]["channelTitle"].as_str().unwrap_or("").to_string();
    let like_count = item["statistics"]["likeCount"].as_str().unwrap_or("0").to_string();
    let view_count = item["statistics"]["viewCount"].as_str().unwrap_or("0").to_string();
    let viewers = item["liveStreamingDetails"]["concurrentViewers"].as_str().unwrap_or("0").to_string();
    let is_live = item["snippet"]["liveBroadcastContent"].as_str() == Some("live");

    Ok(StreamInfo {
        title,
        channel_name,
        viewers,
        like_count,
        view_count,
        is_live,
    })
}
