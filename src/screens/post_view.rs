use crate::app_simple::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let selected_idx = app.topic_posts_list_state.selected().unwrap_or(0);

    if selected_idx >= app.current_topic_posts.len() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Error")
            .border_style(Style::default().fg(Color::Red));
        let paragraph = Paragraph::new("Post not found").block(block);
        frame.render_widget(paragraph, frame.area());
        return;
    }

    let post = &app.current_topic_posts[selected_idx];

    // Check if we have images for this post
    let has_images = app.post_image_urls.contains_key(&post.id);
    let image_height = if has_images { 3 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(image_height),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    // Header
    let header_text = format!(
        "Post #{} by {} - {}",
        post.post_number,
        post.username,
        format_date(&post.created_at)
    );
    let header = Paragraph::new(header_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(header, chunks[0]);

    // Images (if any) - show placeholders for now
    if has_images {
        if let Some(urls) = app.post_image_urls.get(&post.id) {
            render_image_placeholders(frame, chunks[1], urls);
        }
    }

    // Post content
    let text = post.raw.as_deref().unwrap_or_else(|| &post.cooked);
    let text = if post.raw.is_some() {
        text.to_string()
    } else {
        strip_html(text)
    };

    let text = if text.trim().is_empty() {
        "[Post content not available]".to_string()
    } else {
        text
    };

    let text_len = text.len();
    let image_count = app.post_image_urls.get(&post.id).map(|imgs| imgs.len()).unwrap_or(0);
    let title = if image_count > 0 {
        format!("Post Content ({} chars, {} images)", text_len, image_count)
    } else {
        format!("Post Content ({} chars)", text_len)
    };

    let content = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.post_scroll_offset as u16, 0));

    frame.render_widget(content, chunks[2]);

    // Footer
    let footer = Paragraph::new("j/k: scroll | Esc: back to topic | q: quit")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, chunks[3]);
}

fn render_image_placeholders(frame: &mut Frame, area: Rect, urls: &[String]) {
    if urls.is_empty() {
        return;
    }

    // Show placeholder text indicating images are present
    // TODO: Implement actual image rendering with ratatui-image once version compatibility is sorted
    let placeholder_text = urls.iter()
        .map(|url| format!("[ Image: {} ]", url.split('/').last().unwrap_or("unknown")))
        .collect::<Vec<_>>()
        .join("\n");

    let placeholder = Paragraph::new(placeholder_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Images (placeholders)")
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(placeholder, area);
}

fn strip_html(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }

    text.trim().to_string()
}

fn format_date(iso_date: &str) -> String {
    if let Some(date_part) = iso_date.split('T').next() {
        date_part.to_string()
    } else {
        iso_date.to_string()
    }
}
