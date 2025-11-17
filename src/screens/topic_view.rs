use crate::app_simple::App;
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
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

    let content = format!(
        "Title: {}\nAuthor: {}\nCategory: {}\nReplies: {}\n\n(Topic content would go here)\n\nPress Esc to go back",
        topic.title, topic.author, topic.category, topic.replies
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Topic #{}", topic.id))
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(content).block(block);

    frame.render_widget(paragraph, frame.area());
}
