use crate::app_simple::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    let channels: Vec<ListItem> = app
        .chat_channels
        .iter()
        .map(|channel| {
            let title = channel.title.clone();
            let line = Line::from(vec![
                Span::styled(
                    format!("#{} ", title),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let channels_list = List::new(channels)
        .block(
            Block::default()
                .title("Chat Channels")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("➤ ");

    frame.render_stateful_widget(
        channels_list,
        chunks[0],
        &mut app.chat_channels_list_state.clone(),
    );

    let help = Line::from(vec![
        Span::raw("j/k: navigate | Enter: open channel | Esc: back to topics | q: quit"),
    ]);

    frame.render_widget(help, chunks[1]);
}
