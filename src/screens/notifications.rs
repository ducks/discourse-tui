use crate::app_simple::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    // Header
    let unread_count = app.notifications.iter().filter(|n| !n.read).count();
    let header_text = if unread_count > 0 {
        format!("Notifications ({} unread)", unread_count)
    } else {
        "Notifications (all read)".to_string()
    };
    let header = Block::default()
        .borders(Borders::ALL)
        .title(header_text)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(header, chunks[0]);

    // Notifications list
    let items: Vec<ListItem> = app
        .notifications
        .iter()
        .map(|notification| {
            let title = notification
                .fancy_title
                .as_ref()
                .or(notification.data.topic_title.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("(no title)");

            let username = notification
                .data
                .display_username
                .as_ref()
                .or(notification.data.original_username.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("unknown");

            let notification_type = match notification.notification_type {
                1 => "mentioned you",
                2 => "replied to you",
                3 => "quoted you",
                5 => "replied to topic",
                6 => "private message",
                9 => "replied to topic",
                12 => "invited to private message",
                15 => "moved post",
                17 => "mentioned group",
                18 => "watching topic",
                25 => "reacted to post",
                _ => "notification",
            };

            // Format: "username action: title"
            let text = format!("{} {}: {}", username, notification_type, title);

            let style = if notification.read {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            };

            let unread_marker = if notification.read { " " } else { "● " };
            let content = format!("{}{}", unread_marker, text);

            ListItem::new(Span::styled(content, style))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Notification List"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("➤ ");

    frame.render_stateful_widget(list, chunks[1], &mut app.notifications_list_state.clone());

    // Footer
    let footer_text = if app.notifications.is_empty() {
        "No notifications | Esc: back to main | q: quit"
    } else {
        "j/k: scroll | Enter: open topic | Esc: back to main | q: quit"
    };
    let footer = Block::default().title(footer_text);
    frame.render_widget(footer, chunks[2]);
}
