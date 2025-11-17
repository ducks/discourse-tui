use crate::app_simple::{App, ForumPickerMode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    match app.forum_picker_mode {
        ForumPickerMode::List => draw_list(frame, app),
        ForumPickerMode::AddForum => draw_add_forum(frame, app),
    }
}

fn draw_list(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(frame.area());

    let forum_items: Vec<ListItem> = app
        .config
        .forums
        .iter()
        .map(|forum| {
            let selected_marker = if app.config.current.selected.as_ref() == Some(&forum.id) {
                "★ "
            } else {
                "  "
            };
            ListItem::new(Line::from(vec![
                Span::raw(selected_marker),
                Span::styled(&forum.name, Style::default().fg(Color::Cyan)),
                Span::raw(" - "),
                Span::raw(&forum.url),
            ]))
        })
        .collect();

    let list = List::new(forum_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select Forum")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("➤ ");

    frame.render_stateful_widget(list, chunks[0], &mut app.forum_picker_list_state);

    let footer = Paragraph::new("j/k: navigate | Enter: select forum | a: add forum | d: delete forum | 1: main screen | q: quit")
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(footer, chunks[1]);
}

fn draw_add_forum(frame: &mut Frame, app: &mut App) {
    let error_height = if app.add_forum_state.error_message.is_some() { 3 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(9),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(error_height),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let title = Paragraph::new("Add New Forum")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    frame.render_widget(title, chunks[0]);

    let instructions = Paragraph::new(
        "User API Key (recommended):\n\
         1. Visit: [forum-url]/u/[username]/preferences/security\n\
         2. Click 'New API Key' (any user can generate)\n\
         3. Enter description (e.g., 'discourse-tui')\n\n\
         Admin API Key (requires admin):\n\
         Provide both API Key + Username fields"
    )
    .block(Block::default().borders(Borders::ALL).title("Instructions"))
    .style(Style::default().fg(Color::Gray));
    frame.render_widget(instructions, chunks[1]);

    let field_names = ["Forum Name", "Forum URL", "User API Key (optional)", "Admin API Key (optional)", "Username (for admin key)"];
    let field_values = [
        &app.add_forum_state.name,
        &app.add_forum_state.url,
        &app.add_forum_state.user_api_key,
        &app.add_forum_state.api_key,
        &app.add_forum_state.username,
    ];

    for (idx, (name, value)) in field_names.iter().zip(field_values.iter()).enumerate() {
        let is_active = app.add_forum_state.active_field == idx;
        let border_color = if is_active { Color::Cyan } else { Color::White };

        // Mask User API Key (field 2) and Admin API Key (field 3)
        let display_value = if (idx == 2 || idx == 3) && !value.is_empty() {
            "*".repeat(value.len())
        } else {
            value.to_string()
        };

        let input = Paragraph::new(display_value)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(*name)
                    .border_style(Style::default().fg(border_color)),
            );
        frame.render_widget(input, chunks[2 + idx]);
    }

    // Error message display
    if let Some(error) = &app.add_forum_state.error_message {
        let error_widget = Paragraph::new(error.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Error")
                    .border_style(Style::default().fg(Color::Red)),
            )
            .style(Style::default().fg(Color::Red));
        frame.render_widget(error_widget, chunks[7]);
    }

    let footer = Paragraph::new("Tab: next field | Enter: save | Esc: cancel | Type to enter text")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, chunks[9]);
}
