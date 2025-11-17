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
        Screen::ForumPicker => draw_forum_picker(f, app),
        Screen::AddForum => draw_add_forum(f, app),
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

fn draw_forum_picker(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(f.area());

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

    f.render_stateful_widget(list, chunks[0], &mut app.forum_picker_state.clone());

    let footer = Paragraph::new("j/k: navigate | Enter: select forum | a: add forum | d: delete forum | q: quit")
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(footer, chunks[1]);
}

fn draw_add_forum(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Add New Forum")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(title, chunks[0]);

    // Instructions
    let instructions = Paragraph::new(
        "To get your API credentials:\n\
         1. Visit: [forum-url]/u/[username]/preferences/security\n\
         2. Click 'New API Key'\n\
         3. Enter a description (e.g., 'discourse-tui')\n\
         4. Copy your key\n\n\
         Leave API key blank for read-only access to public content."
    )
    .block(Block::default().borders(Borders::ALL).title("Instructions"))
    .style(Style::default().fg(Color::Gray));
    f.render_widget(instructions, chunks[1]);

    // Input fields
    let field_names = ["Forum Name", "Forum URL", "API Key (optional)", "Username (optional)"];
    let field_values = [
        &app.add_forum_inputs.name,
        &app.add_forum_inputs.url,
        &app.add_forum_inputs.api_key,
        &app.add_forum_inputs.username,
    ];

    for (idx, (name, value)) in field_names.iter().zip(field_values.iter()).enumerate() {
        let is_active = app.add_forum_inputs.active_field == idx;
        let border_color = if is_active { Color::Cyan } else { Color::White };

        let display_value = if idx == 2 && !value.is_empty() {
            // Mask API key
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
        f.render_widget(input, chunks[2 + idx]);
    }

    let footer = Paragraph::new("Tab: next field | Enter: save | Esc: cancel | Type to enter text")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[7]);
}
