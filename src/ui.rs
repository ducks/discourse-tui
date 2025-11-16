use crate::app::{App, PaneFocus, Screen};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::TopicList => draw_topic_list(f, app),
        Screen::TopicView => draw_topic_view(f, app),
    }
}

fn draw_topic_list(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(f.area());

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
                .title("Discourse")
                .border_style(Style::default().fg(sidebar_border_color)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("➤ ");

    f.render_stateful_widget(sidebar, chunks[0], &mut app.sidebar_state.clone());

    // Topics list
    let topics_items: Vec<ListItem> = app
        .topics
        .iter()
        .map(|topic| {
            let unread_indicator = if topic.unread { "● " } else { "  " };
            let line = Line::from(vec![
                Span::styled(unread_indicator, Style::default().fg(Color::Blue)),
                Span::raw(&topic.title),
                Span::styled(
                    format!(" ({})", topic.replies),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let topics_border_color = if app.focus == PaneFocus::Topics {
        Color::Cyan
    } else {
        Color::White
    };

    let topics_title = match app.current_filter {
        crate::app::ViewFilter::AllTopics => "Topics",
        crate::app::ViewFilter::MyPosts => "My Posts",
        crate::app::ViewFilter::MyMessages => "My Messages",
        crate::app::ViewFilter::Category(_) => "Category Topics",
        crate::app::ViewFilter::Tag(_) => "Tagged Topics",
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

    f.render_stateful_widget(topics, chunks[1], &mut app.topics_state.clone());

    // Footer with keybinding hints
    let footer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    let footer = Paragraph::new("j/k: navigate | Tab: switch pane | Enter: open topic | q: quit")
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(footer, footer_chunks[1]);
}

fn draw_topic_view(f: &mut Frame, app: &App) {
    let selected_topic_idx = app.topics_state.selected().unwrap_or(0);
    let topic = &app.topics[selected_topic_idx];

    let content = format!(
        "Title: {}\nAuthor: {}\nCategory: {}\nReplies: {}\n\n(Topic content would go here)\n\nPress Esc to go back",
        topic.title, topic.author, topic.category, topic.replies
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Topic #{}", topic.id))
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(content).block(block);

    f.render_widget(paragraph, f.area());
}
