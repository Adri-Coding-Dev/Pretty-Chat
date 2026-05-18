//! Motor de renderizado de la interfaz de usuario.
//! Dibuja los paneles de cabecera, chat y estadísticas aplicando el tema seleccionado.

use ratatui::{
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Paragraph, Wrap},
    Frame,
};
use crate::app::App;
use crate::models::{Badge, ChatMessage};
use crate::themes::parse_hex;

/// Convierte un color hexadecimal (#RRGGBB) a Color de ratatui.
fn get_style_from_hex(hex: &str) -> Color {
    if let Some((r, g, b)) = parse_hex(hex) {
        Color::Rgb(r, g, b)
    } else {
        Color::White
    }
}

/// Función principal de dibujado.
pub fn draw(f: &mut Frame, app: &App) {
    let theme = app.theme.as_ref();

    // Extraer colores del tema o usar valores por defecto
    let bg = theme.map(|t| get_style_from_hex(&t.colors.background)).unwrap_or(Color::Black);
    let fg = theme.map(|t| get_style_from_hex(&t.colors.foreground)).unwrap_or(Color::White);
    let border_color = theme.map(|t| get_style_from_hex(&t.colors.border)).unwrap_or(Color::Gray);
    let mod_color = theme.map(|t| get_style_from_hex(&t.colors.moderator)).unwrap_or(Color::Cyan);
    let verified_color = theme.map(|t| get_style_from_hex(&t.colors.verified)).unwrap_or(Color::LightBlue);
    let member_color = theme.map(|t| get_style_from_hex(&t.colors.member)).unwrap_or(Color::Green);
    let superchat_color = theme.map(|t| get_style_from_hex(&t.colors.superchat)).unwrap_or(Color::Red);
    let info_color = theme.map(|t| get_style_from_hex(&t.colors.info)).unwrap_or(Color::LightCyan);
    let alert_color = theme.map(|t| get_style_from_hex(&t.colors.alert)).unwrap_or(Color::LightRed);

    // Layout vertical: cabecera, chat, pie
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Cabecera
            Constraint::Min(10),     // Chat
            Constraint::Length(3),   // Estadísticas
        ])
        .split(f.size());

    // ---------- Cabecera ----------
    let header_text = if let Some(ref info) = app.stream_info {
        format!(
            " {} {} | {} | {} likes | {} views | {} watching ",
            if info.is_live { "LIVE" } else { "OFFLINE" },
            info.title,
            info.channel_name,
            info.like_count,
            info.view_count,
            info.viewers
        )
    } else {
        " pchat — Live Chat Viewer ".to_string()
    };

    let header = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(info_color))
        .title(format!(" {} ", header_text))
        .title_style(Style::default().fg(info_color).bold())
        .style(Style::default().bg(bg));
    f.render_widget(header, main_chunks[0]);

    // ---------- Chat ----------
    let chat_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(" LIVE CHAT ")
        .title_style(Style::default().fg(alert_color).bold())
        .style(Style::default().bg(bg));
    let chat_area = chat_block.inner(main_chunks[1]);
    f.render_widget(chat_block, main_chunks[1]);

    let visible_messages = app.visible_messages(chat_area.height as usize);

    // Formatear mensajes y agregar separadores
    let message_lines: Vec<Line> = visible_messages
        .iter()
        .map(|msg| format_message(msg, app.compact, mod_color, verified_color, member_color, superchat_color, fg))
        .collect();

    // Línea separadora fina (guiones)
    let separator = "─".repeat(chat_area.width as usize);
    let separator_line = Line::from(Span::styled(
        separator,
        Style::default().fg(border_color),
    ));

    let mut final_lines = Vec::new();
    for (i, line) in message_lines.into_iter().enumerate() {
        if i > 0 {
            final_lines.push(separator_line.clone());
        }
        final_lines.push(line);
    }

    let paragraph = Paragraph::new(final_lines)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chat_area);

    // ---------- Pie de estadísticas ----------
    let stats_text = format!(
        " msgs/s: {:.1} | spam: {:.0}% | queue: {} | total: {} ",
        app.messages_per_second(),
        app.spam_ratio() * 100.0,
        app.queue_len(),
        app.total_messages
    );

    let stats_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(info_color))
        .title(" STATS ")
        .title_style(Style::default().fg(info_color).bold())
        .style(Style::default().bg(bg));
    f.render_widget(stats_block, main_chunks[2]);

    let inner_stats = main_chunks[2].inner(Margin { vertical: 1, horizontal: 1 });
    let stats_paragraph = Paragraph::new(stats_text)
        .style(Style::default().fg(fg).bold());
    f.render_widget(stats_paragraph, inner_stats);
}

/// Convierte un mensaje en una línea de texto formateada.
fn format_message(
    msg: &ChatMessage,
    compact: bool,
    mod_color: Color,
    verified_color: Color,
    member_color: Color,
    superchat_color: Color,
    default_fg: Color,
) -> Line<'static> {
    let mut spans = vec![];

    if msg.is_superchat {
        let amount = msg.superchat_amount.clone().unwrap_or_else(|| "$".to_string());
        spans.push(Span::styled(
            format!("[{}] ", amount),
            Style::default().fg(superchat_color).bold(),
        ));
    }

    if !compact {
        for badge in &msg.badges {
            match badge {
                Badge::Moderator => spans.push(Span::styled("[MOD] ", Style::default().fg(mod_color).bold())),
                Badge::Verified => spans.push(Span::styled("[VERIF] ", Style::default().fg(verified_color))),
                Badge::Member => spans.push(Span::styled("[MEM] ", Style::default().fg(member_color).italic())),
                Badge::Custom(s) => spans.push(Span::raw(format!("[{}] ", s))),
            }
        }
    }

    let name_color = msg.color.map(|c| Color::Rgb(c.r, c.g, c.b))
        .unwrap_or(default_fg);
    spans.push(Span::styled(msg.username.clone(), Style::default().fg(name_color).bold()));

    if !compact {
        // Convertir timestamp a HH:MM:SS o MM:SS
        let ts = msg.timestamp as f64 / 1000.0;
        let secs = ts as u64 % 60;
        let mins = (ts as u64 / 60) % 60;
        let hours = ts as u64 / 3600;
        let time_str = if hours > 0 {
            format!(" {:02}:{:02}:{:02}", hours, mins, secs)
        } else {
            format!(" {:02}:{:02}", mins, secs)
        };
        spans.push(Span::raw(time_str));
    }

    spans.push(Span::raw(": "));
    spans.push(Span::raw(msg.content.clone()));

    Line::from(spans)
}
