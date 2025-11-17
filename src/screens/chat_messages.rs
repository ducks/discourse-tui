use crate::app_simple::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let composer_height = if app.chat_composer_visible { 5 } else { 0 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),              // Header
            Constraint::Min(0),                 // Messages
            Constraint::Length(composer_height), // Composer (hidden or visible)
            Constraint::Length(1),              // Help
        ])
        .split(frame.area());

    let channel_name = app
        .chat_channels
        .iter()
        .find(|c| Some(c.id) == app.selected_channel_id)
        .map(|c| c.title.clone())
        .unwrap_or_else(|| "Unknown Channel".to_string());

    let header = Block::default()
        .title(format!("Chat: #{}", channel_name))
        .borders(Borders::ALL);

    frame.render_widget(header, chunks[0]);

    let messages: Vec<ListItem> = app
        .current_chat_messages
        .iter()
        .map(|message| {
            let username = &message.user.username;
            let text = &message.message;
            let timestamp = &message.created_at;

            let max_width = chunks[1].width.saturating_sub(4) as usize;
            let wrapped = wrap_text(text, max_width);

            let mut lines = vec![Line::from(vec![
                Span::styled(
                    format!("{} ", username),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format_date(timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
            ])];

            for line in wrapped.split('\n') {
                lines.push(Line::from(line.to_string()));
            }

            lines.push(Line::from(""));

            ListItem::new(lines)
        })
        .collect();

    let messages_list = List::new(messages)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(
        messages_list,
        chunks[1],
        &mut app.chat_messages_list_state.clone(),
    );

    // Composer input (only render if visible)
    if app.chat_composer_visible {
        let (composer_style, composer_title) = if app.chat_composer_insert_mode {
            (
                Style::default().fg(Color::Yellow),
                "Message -- INSERT -- (Esc: normal mode, Enter: newline)"
            )
        } else {
            (
                Style::default().fg(Color::Green),
                "Message -- NORMAL -- (i: insert, Enter: send, Esc: hide)"
            )
        };

        let composer = ratatui::widgets::Paragraph::new(app.chat_composer_input.as_str())
            .block(
                Block::default()
                    .title(composer_title)
                    .borders(Borders::ALL)
                    .border_style(composer_style),
            )
            .wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(composer, chunks[2]);
    }

    let help = Line::from(vec![Span::raw(
        "j/k: scroll | SPACE/i: compose | r: refresh | Esc: back | q: quit",
    )]);

    frame.render_widget(help, chunks[3]);
}

fn wrap_text(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return text.to_string();
    }

    let mut result = String::new();
    for paragraph in text.split('\n') {
        let mut current_line = String::new();
        for word in paragraph.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                result.push_str(&current_line);
                result.push('\n');
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            result.push_str(&current_line);
        }
        result.push('\n');
    }

    result
}

fn format_date(iso_date: &str) -> String {
    iso_date
        .split('T')
        .next()
        .map(|d| d.to_string())
        .unwrap_or_else(|| iso_date.to_string())
}
