mod app_simple;
mod config;
mod screens;

use app_simple::{App, AppScreen, ForumPickerMode, PaneFocus, SidebarItem, ViewFilter};
use config::Forum;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use discourse_api_rs::DiscourseClient;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let res = run_app(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

async fn load_topic_posts(app: &mut App, topic_id: u64) -> Result<(), Box<dyn std::error::Error>> {
    let forum = app
        .config
        .get_current_forum()
        .ok_or("No forum selected")?;

    let url = if !forum.url.starts_with("http://") && !forum.url.starts_with("https://") {
        format!("https://{}", forum.url)
    } else {
        forum.url.clone()
    };

    let client = if let (Some(api_key), Some(username)) = (&forum.api_key, &forum.username) {
        DiscourseClient::with_api_key(&url, api_key, username)
    } else {
        DiscourseClient::new(&url)
    };

    let topic_response = client.get_topic(topic_id).await?;
    app.current_topic_posts = topic_response.post_stream.posts;
    app.topic_posts_list_state.select(Some(0));

    Ok(())
}

async fn load_forum_data(app: &mut App, forum: &Forum) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure URL has protocol
    let url = if !forum.url.starts_with("http://") && !forum.url.starts_with("https://") {
        format!("https://{}", forum.url)
    } else {
        forum.url.clone()
    };

    let client = if let (Some(api_key), Some(username)) = (&forum.api_key, &forum.username) {
        DiscourseClient::with_api_key(&url, api_key, username)
    } else {
        DiscourseClient::new(&url)
    };

    let latest = client.get_latest().await?;
    let categories = client.get_categories().await?;

    // Convert API topics to TUI topics
    app.all_topics = latest
        .topic_list
        .topics
        .iter()
        .map(|api_topic| {
            let category_name = api_topic
                .category_id
                .and_then(|cat_id| {
                    categories
                        .iter()
                        .find(|c| c.id == cat_id)
                        .map(|c| c.name.clone())
                })
                .unwrap_or_else(|| "uncategorized".to_string());

            let author = api_topic
                .posters
                .first()
                .and_then(|p| {
                    latest
                        .users
                        .iter()
                        .find(|u| u.id == p.user_id)
                        .map(|u| u.username.clone())
                })
                .unwrap_or_else(|| "unknown".to_string());

            app_simple::Topic {
                id: api_topic.id as usize,
                title: api_topic.title.clone(),
                author,
                category: category_name,
                replies: api_topic.reply_count as usize,
                unread: false,
            }
        })
        .collect();

    app.topics = app.all_topics.clone();
    app.categories = categories.clone();

    // Build sidebar
    let mut sidebar_items = vec![
        SidebarItem {
            label: "Topics".to_string(),
            unread_count: None,
            color: None,
            filter: Some(ViewFilter::AllTopics),
        },
        SidebarItem {
            label: "My posts".to_string(),
            unread_count: None,
            color: None,
            filter: Some(ViewFilter::MyPosts),
        },
        SidebarItem {
            label: "My messages".to_string(),
            unread_count: None,
            color: None,
            filter: Some(ViewFilter::MyMessages),
        },
        SidebarItem {
            label: "New Topic".to_string(),
            unread_count: None,
            color: None,
            filter: None,
        },
        SidebarItem {
            label: "--- CATEGORIES ---".to_string(),
            unread_count: None,
            color: None,
            filter: None,
        },
    ];

    for (idx, category) in categories.iter().take(10).enumerate() {
        let color = parse_color(&category.color);
        sidebar_items.push(SidebarItem {
            label: format!("■ {}", category.name),
            unread_count: None,
            color,
            filter: Some(ViewFilter::Category(idx)),
        });
    }

    sidebar_items.push(SidebarItem {
        label: "All categories".to_string(),
        unread_count: None,
        color: None,
        filter: Some(ViewFilter::AllTopics),
    });

    app.sidebar_items = sidebar_items;
    app.current_filter = ViewFilter::AllTopics;

    // Reset states
    app.sidebar_state.select(Some(0));
    app.topics_state.select(Some(0));
    app.focus = PaneFocus::Sidebar;

    Ok(())
}

fn parse_color(hex: &str) -> Option<ratatui::style::Color> {
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(ratatui::style::Color::Rgb(r, g, b))
    } else {
        None
    }
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            match app.screen {
                AppScreen::ForumPicker => screens::forum_picker::draw(f, app),
                AppScreen::MainScreen => screens::main_screen::draw(f, app),
                AppScreen::TopicView => screens::topic_view::draw(f, app),
            }
        })?;

        if let Event::Key(key) = event::read()? {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return Ok(());
            }

            match app.screen {
                AppScreen::ForumPicker => handle_forum_picker_input(key.code, app).await?,
                AppScreen::MainScreen => handle_main_screen_input(key.code, app).await?,
                AppScreen::TopicView => handle_topic_view_input(key.code, app)?,
            }
        }
    }
}

async fn handle_forum_picker_input(key: KeyCode, app: &mut App) -> io::Result<()> {
    match app.forum_picker_mode {
        ForumPickerMode::List => match key {
            KeyCode::Char('q') => std::process::exit(0),
            KeyCode::Char('1') => {
                // Only go to main screen if we have a selected forum
                if app.config.current.selected.is_some() {
                    app.goto_screen(AppScreen::MainScreen);
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let len = app.config.forums.len();
                if len > 0 {
                    let i = app.forum_picker_list_state.selected().unwrap_or(0);
                    app.forum_picker_list_state.select(Some(if i >= len - 1 { 0 } else { i + 1 }));
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let len = app.config.forums.len();
                if len > 0 {
                    let i = app.forum_picker_list_state.selected().unwrap_or(0);
                    app.forum_picker_list_state.select(Some(if i == 0 { len - 1 } else { i - 1 }));
                }
            }
            KeyCode::Char('a') => {
                app.forum_picker_mode = ForumPickerMode::AddForum;
                app.add_forum_state = Default::default();
            }
            KeyCode::Char('d') => {
                if let Some(idx) = app.forum_picker_list_state.selected() {
                    if let Some(forum) = app.config.forums.get(idx).cloned() {
                        app.config.remove_forum(&forum.id);
                        let _ = app.config.save();

                        if app.config.forums.is_empty() {
                            app.forum_picker_mode = ForumPickerMode::AddForum;
                        } else if idx >= app.config.forums.len() {
                            app.forum_picker_list_state.select(Some(app.config.forums.len() - 1));
                        }
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(idx) = app.forum_picker_list_state.selected() {
                    if let Some(forum) = app.config.forums.get(idx).cloned() {
                        app.config.set_current_forum(forum.id.clone());
                        let _ = app.config.save();

                        // Load forum data
                        if let Err(e) = load_forum_data(app, &forum).await {
                            eprintln!("Failed to load forum data: {}", e);
                        } else {
                            app.goto_screen(AppScreen::MainScreen);
                        }
                    }
                }
            }
            _ => {}
        },
        ForumPickerMode::AddForum => match key {
            KeyCode::Char('q') => std::process::exit(0),
            KeyCode::Char(c) => {
                let field = match app.add_forum_state.active_field {
                    0 => &mut app.add_forum_state.name,
                    1 => &mut app.add_forum_state.url,
                    2 => &mut app.add_forum_state.api_key,
                    3 => &mut app.add_forum_state.username,
                    _ => return Ok(()),
                };
                field.push(c);
            }
            KeyCode::Backspace => {
                let field = match app.add_forum_state.active_field {
                    0 => &mut app.add_forum_state.name,
                    1 => &mut app.add_forum_state.url,
                    2 => &mut app.add_forum_state.api_key,
                    3 => &mut app.add_forum_state.username,
                    _ => return Ok(()),
                };
                field.pop();
            }
            KeyCode::Tab => {
                app.add_forum_state.active_field = (app.add_forum_state.active_field + 1) % 4;
            }
            KeyCode::Enter => {
                if !app.add_forum_state.name.is_empty() && !app.add_forum_state.url.is_empty() {
                    let id = app.add_forum_state.name.to_lowercase().replace(' ', "-");
                    let forum = Forum {
                        id: id.clone(),
                        name: app.add_forum_state.name.clone(),
                        url: app.add_forum_state.url.clone(),
                        api_key: if app.add_forum_state.api_key.is_empty() {
                            None
                        } else {
                            Some(app.add_forum_state.api_key.clone())
                        },
                        username: if app.add_forum_state.username.is_empty() {
                            None
                        } else {
                            Some(app.add_forum_state.username.clone())
                        },
                    };
                    app.config.add_forum(forum.clone());
                    app.config.set_current_forum(id);
                    let _ = app.config.save();

                    // Load forum data
                    if let Err(e) = load_forum_data(app, &forum).await {
                        eprintln!("Failed to load forum data: {}", e);
                    } else {
                        app.goto_screen(AppScreen::MainScreen);
                    }
                }
            }
            KeyCode::Esc => {
                if !app.config.forums.is_empty() {
                    app.forum_picker_mode = ForumPickerMode::List;
                }
            }
            _ => {}
        },
    }
    Ok(())
}

async fn handle_main_screen_input(key: KeyCode, app: &mut App) -> io::Result<()> {
    match key {
        KeyCode::Char('q') => std::process::exit(0),
        KeyCode::Char('5') => {
            app.goto_screen(AppScreen::ForumPicker);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let (list_state, list_len) = match app.focus {
                PaneFocus::Sidebar => (&mut app.sidebar_state, app.sidebar_items.len()),
                PaneFocus::Topics => (&mut app.topics_state, app.topics.len()),
            };

            if list_len > 0 {
                let i = list_state.selected().unwrap_or(0);
                list_state.select(Some(if i >= list_len - 1 { 0 } else { i + 1 }));
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let (list_state, list_len) = match app.focus {
                PaneFocus::Sidebar => (&mut app.sidebar_state, app.sidebar_items.len()),
                PaneFocus::Topics => (&mut app.topics_state, app.topics.len()),
            };

            if list_len > 0 {
                let i = list_state.selected().unwrap_or(0);
                list_state.select(Some(if i == 0 { list_len - 1 } else { i - 1 }));
            }
        }
        KeyCode::Tab => {
            app.focus = match app.focus {
                PaneFocus::Sidebar => PaneFocus::Topics,
                PaneFocus::Topics => PaneFocus::Sidebar,
            };
        }
        KeyCode::Enter => {
            if app.focus == PaneFocus::Sidebar {
                if let Some(idx) = app.sidebar_state.selected() {
                    if let Some(filter) = app.sidebar_items.get(idx).and_then(|item| item.filter) {
                        apply_filter(app, filter);
                    }
                }
            } else {
                app.selected_topic_idx = app.topics_state.selected().unwrap_or(0);
                if let Some(topic) = app.topics.get(app.selected_topic_idx) {
                    if let Err(e) = load_topic_posts(app, topic.id as u64).await {
                        eprintln!("Failed to load topic posts: {}", e);
                    } else {
                        app.goto_screen(AppScreen::TopicView);
                    }
                }
            }
        }
        KeyCode::Esc => {
            app.goto_screen(AppScreen::ForumPicker);
        }
        _ => {}
    }
    Ok(())
}

fn handle_topic_view_input(key: KeyCode, app: &mut App) -> io::Result<()> {
    match key {
        KeyCode::Char('q') => std::process::exit(0),
        KeyCode::Esc => {
            app.goto_screen(AppScreen::MainScreen);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let len = app.current_topic_posts.len();
            if len > 0 {
                let i = app.topic_posts_list_state.selected().unwrap_or(0);
                app.topic_posts_list_state.select(Some(if i >= len - 1 { 0 } else { i + 1 }));
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let len = app.current_topic_posts.len();
            if len > 0 {
                let i = app.topic_posts_list_state.selected().unwrap_or(0);
                app.topic_posts_list_state.select(Some(if i == 0 { len - 1 } else { i - 1 }));
            }
        }
        _ => {}
    }
    Ok(())
}

fn apply_filter(app: &mut App, filter: ViewFilter) {
    app.current_filter = filter;
    app.topics = match filter {
        ViewFilter::AllTopics => app.all_topics.clone(),
        ViewFilter::MyPosts => app
            .all_topics
            .iter()
            .filter(|t| t.author == "ducks")
            .cloned()
            .collect(),
        ViewFilter::MyMessages => vec![],
        ViewFilter::Category(cat_idx) => {
            if let Some(category) = app.categories.get(cat_idx) {
                let category_name = &category.name;
                app.all_topics
                    .iter()
                    .filter(|t| &t.category == category_name)
                    .cloned()
                    .collect()
            } else {
                app.all_topics.clone()
            }
        }
        ViewFilter::Tag(_) => app.all_topics.clone(),
    };

    if !app.topics.is_empty() {
        app.topics_state.select(Some(0));
    }
}
