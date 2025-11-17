use crate::app_simple::{App, PaneFocus, ViewFilter};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(frame.area());

    // Sidebar
    let sidebar_items: Vec<ListItem> = app
        .sidebar_items
        .iter()
        .map(|item| {
            let mut spans = vec![];

            if let Some(color) = item.color {
                spans.push(Span::styled("", Style::default().fg(color)));
            }

            spans.push(Span::raw(&item.label));

            if let Some(count) = item.unread_count {
                spans.push(Span::raw(format!(" ({})", count)));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let sidebar_border_color = if app.focus == PaneFocus::Sidebar {
        Color::Cyan
    } else {
        Color::White
    };

    let sidebar = List::new(sidebar_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Navigation")
                .border_style(Style::default().fg(sidebar_border_color)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("➤ ");

    frame.render_stateful_widget(sidebar, chunks[0], &mut app.sidebar_state);

    // Topics
    let topics_title = match app.current_filter {
        ViewFilter::AllTopics => "All Topics",
        ViewFilter::MyPosts => "My Posts",
        ViewFilter::MyMessages => "My Messages",
        ViewFilter::Category(_) => "Category Topics",
        ViewFilter::Tag(_) => "Tagged Topics",
    };

    let topics_items: Vec<ListItem> = app
        .topics
        .iter()
        .map(|topic| {
            let unread_marker = if topic.unread { "● " } else { "  " };

            ListItem::new(Line::from(vec![
                Span::raw(unread_marker),
                Span::styled(&topic.title, Style::default().fg(Color::White)),
                Span::raw(format!(" ({})", topic.replies)),
            ]))
        })
        .collect();

    let topics_border_color = if app.focus == PaneFocus::Topics {
        Color::Cyan
    } else {
        Color::White
    };

    let topics = List::new(topics_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(topics_title)
                .border_style(Style::default().fg(topics_border_color)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("➤ ");

    frame.render_stateful_widget(topics, chunks[1], &mut app.topics_state);

    // Footer
    let footer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let footer = Paragraph::new("j/k: navigate | Tab: switch pane | Enter: open topic | 5: forum picker | q: quit")
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(footer, footer_chunks[1]);
}
