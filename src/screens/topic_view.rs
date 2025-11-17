use crate::app_simple::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    if app.selected_topic_idx >= app.topics.len() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Error")
            .border_style(Style::default().fg(Color::Red));

        let paragraph = Paragraph::new("Topic not found").block(block);
        frame.render_widget(paragraph, frame.area());
        return;
    }

    let topic = &app.topics[app.selected_topic_idx];

    // Create header and content areas
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    // Header
    let header_text = format!(
        "{} - {} replies",
        topic.title, topic.replies
    );
    let header = Paragraph::new(header_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(header, chunks[0]);

    // Posts
    if app.current_topic_posts.is_empty() {
        let loading = Paragraph::new("Loading posts...")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(loading, chunks[1]);
    } else {
        // Debug: Check if posts have content
        let empty_posts = app.current_topic_posts.iter().filter(|p| {
            p.raw.as_ref().map_or(true, |r| r.trim().is_empty()) &&
            p.cooked.trim().is_empty()
        }).count();

        if empty_posts > 0 {
            eprintln!("Warning: {} posts have no content in topic {}", empty_posts, topic.id);
        }
        let post_items: Vec<ListItem> = app
            .current_topic_posts
            .iter()
            .map(|post| {
                // Use raw markdown if available, otherwise strip HTML from cooked
                let text = post.raw.as_deref().unwrap_or_else(|| &post.cooked);
                let text = if post.raw.is_some() {
                    text.to_string()
                } else {
                    strip_html(text)
                };

                // Fallback for empty posts
                let text = if text.trim().is_empty() {
                    "[Post content not available]".to_string()
                } else {
                    text
                };

                // Limit text length to prevent extremely long posts from filling screen
                let preview_text = if text.len() > 1000 {
                    format!("{}...\n\n[Press SPACE/ENTER to view full post - {} chars total]", &text[..1000], text.len())
                } else {
                    text
                };

                // Wrap text to fit screen width (accounting for borders and padding)
                let max_width = chunks[1].width.saturating_sub(6) as usize;
                let wrapped = wrap_text(&preview_text, max_width);

                let mut lines = vec![
                    Line::from(vec![
                        Span::styled(
                            format!("#{} ", post.post_number),
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            &post.username,
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::raw(format!(" - {}", format_date(&post.created_at))),
                    ]),
                    Line::from(""),
                ];

                // Add wrapped text lines
                for line in wrapped {
                    lines.push(Line::from(line));
                }

                lines.push(Line::from(""));

                ListItem::new(lines)
            })
            .collect();

        let posts = List::new(post_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Posts ({}/{})", app.current_topic_posts.len(), topic.replies + 1)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("➤ ");

        frame.render_stateful_widget(posts, chunks[1], &mut app.topic_posts_list_state);
    }

    // Footer
    let footer = Paragraph::new("j/k: scroll | SPACE/ENTER: view post | Esc: back | q: quit")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, chunks[2]);
}

fn strip_html(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    let mut chars = html.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }

    text.trim().to_string()
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();

    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current_line = String::new();
        let mut current_width = 0;

        for word in paragraph.split_whitespace() {
            let word_len = word.len();

            if current_width == 0 {
                // First word on line
                if word_len > max_width {
                    // Word is longer than max_width, split it
                    lines.push(word[..max_width].to_string());
                    current_line = word[max_width..].to_string();
                    current_width = current_line.len();
                } else {
                    current_line = word.to_string();
                    current_width = word_len;
                }
            } else if current_width + 1 + word_len <= max_width {
                // Word fits on current line
                current_line.push(' ');
                current_line.push_str(word);
                current_width += 1 + word_len;
            } else {
                // Word doesn't fit, start new line
                lines.push(current_line);
                if word_len > max_width {
                    lines.push(word[..max_width].to_string());
                    current_line = word[max_width..].to_string();
                    current_width = current_line.len();
                } else {
                    current_line = word.to_string();
                    current_width = word_len;
                }
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }

    lines
}

fn format_date(iso_date: &str) -> String {
    if let Some(date_part) = iso_date.split('T').next() {
        date_part.to_string()
    } else {
        iso_date.to_string()
    }
}
