mod app_simple;
mod config;
mod screens;

use app_simple::{App, AppScreen, ForumPickerMode, PaneFocus, SidebarItem, ViewFilter};
use config::Forum;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
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

    // Enable keyboard enhancement flags for better modifier key detection
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
        )
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let res = run_app(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        PopKeyboardEnhancementFlags
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

    // Always select first post (0) to ensure visibility
    app.topic_posts_list_state.select(Some(0));

    Ok(())
}

async fn load_chat_channels(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
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
        return Err("Chat requires API authentication".into());
    };

    let response = client.get_user_channels().await?;

    // Combine public and DM channels
    let mut all_channels = Vec::new();
    if let Some(public) = response.public_channels {
        all_channels.extend(public);
    }
    if let Some(dms) = response.direct_message_channels {
        all_channels.extend(dms);
    }

    app.chat_channels = all_channels;

    if !app.chat_channels.is_empty() {
        app.chat_channels_list_state.select(Some(0));
    }

    Ok(())
}

async fn send_chat_message(
    app: &mut App,
    channel_id: u64,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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
        return Err("Chat requires API authentication".into());
    };

    client.send_chat_message(channel_id, message).await?;

    Ok(())
}

async fn load_chat_messages(app: &mut App, channel_id: u64) -> Result<(), Box<dyn std::error::Error>> {
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
        return Err("Chat requires API authentication".into());
    };

    let response = client.get_channel_messages(channel_id).await?;
    app.current_chat_messages = response.messages;
    app.selected_channel_id = Some(channel_id);
    app.last_message_poll = std::time::Instant::now();

    if !app.current_chat_messages.is_empty() {
        app.chat_messages_list_state.select(Some(0));
    }

    Ok(())
}

async fn create_post_reply(
    app: &mut App,
    topic_id: u64,
    message: &str,
    reply_to_post_number: Option<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
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
        return Err("Posting requires API authentication".into());
    };

    client.create_post(topic_id, message, reply_to_post_number).await?;

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
                        .find(|u| u.id as i64 == p.user_id)
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
                AppScreen::PostView => screens::post_view::draw(f, app),
                AppScreen::ChatChannels => screens::chat_channels::draw(f, app),
                AppScreen::ChatMessages => screens::chat_messages::draw(f, app),
            }
        })?;

        // Auto-refresh chat messages every 5 seconds
        if app.screen == AppScreen::ChatMessages {
            if app.last_message_poll.elapsed() >= std::time::Duration::from_secs(5) {
                if let Some(channel_id) = app.selected_channel_id {
                    let _ = load_chat_messages(app, channel_id).await;
                }
            }
        }

        // Use poll with timeout so we can auto-refresh
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    return Ok(());
                }

                match app.screen {
                    AppScreen::ForumPicker => handle_forum_picker_input(key, app).await?,
                    AppScreen::MainScreen => handle_main_screen_input(key, app).await?,
                    AppScreen::TopicView => handle_topic_view_input(key, app).await?,
                    AppScreen::PostView => handle_post_view_input(key, app)?,
                    AppScreen::ChatChannels => handle_chat_channels_input(key, app).await?,
                    AppScreen::ChatMessages => handle_chat_messages_input(key, app).await?,
                }
            }
        }
    }
}

async fn handle_forum_picker_input(key: event::KeyEvent, app: &mut App) -> io::Result<()> {
    match app.forum_picker_mode {
        ForumPickerMode::List => match key.code {
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
        ForumPickerMode::AddForum => match key.code {
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
                        eprintln!("Failed to load forum data: {:?}", e);
                        eprintln!("Forum: url={}, has_api_key={}, has_username={}",
                            forum.url,
                            forum.api_key.is_some(),
                            forum.username.is_some()
                        );
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

async fn handle_main_screen_input(key: event::KeyEvent, app: &mut App) -> io::Result<()> {
    match key.code {
        KeyCode::Char('q') => std::process::exit(0),
        KeyCode::Char('2') => {
            // Load chat channels and go to chat view
            if let Err(e) = load_chat_channels(app).await {
                eprintln!("Failed to load chat channels: {}", e);
            } else {
                app.goto_screen(AppScreen::ChatChannels);
            }
        }
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
                        if let Err(e) = apply_filter(app, filter).await {
                            eprintln!("Failed to apply filter: {}", e);
                        }
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

async fn handle_topic_view_input(key: event::KeyEvent, app: &mut App) -> io::Result<()> {
    if app.topic_composer_visible && app.topic_composer_insert_mode {
        // INSERT MODE: Can type
        match key.code {
            KeyCode::Esc => {
                // Exit insert mode, back to normal mode
                app.topic_composer_insert_mode = false;
            }
            KeyCode::Enter => {
                // Enter adds newline in insert mode
                app.topic_composer_input.push('\n');
            }
            KeyCode::Backspace => {
                app.topic_composer_input.pop();
            }
            KeyCode::Char(c) => {
                app.topic_composer_input.push(c);
            }
            _ => {}
        }
    } else if app.topic_composer_visible {
        // NORMAL MODE: Composer visible but can't type
        match key.code {
            KeyCode::Char('i') => {
                // Enter insert mode
                app.topic_composer_insert_mode = true;
            }
            KeyCode::Enter => {
                // Send message in normal mode
                if !app.topic_composer_input.trim().is_empty() {
                    let topic_id = app.topics.get(app.selected_topic_idx).map(|t| t.id as u64);
                    if let Some(topic_id) = topic_id {
                        let message = app.topic_composer_input.clone();
                        let reply_to = app.topic_reply_to_post_number;

                        if let Err(e) = create_post_reply(app, topic_id, &message, reply_to).await {
                            eprintln!("Failed to send reply: {}", e);
                        } else {
                            app.topic_composer_input.clear();
                            app.topic_composer_visible = false;
                            app.topic_composer_insert_mode = false;
                            app.topic_reply_to_post_number = None;
                            // Refresh topic to show our new post
                            let _ = load_topic_posts(app, topic_id).await;
                        }
                    }
                }
            }
            KeyCode::Esc => {
                // Hide composer
                app.topic_composer_visible = false;
                app.topic_composer_insert_mode = false;
            }
            _ => {}
        }
    } else if app.topic_visual_mode {
        // Handle visual mode
        match key.code {
            KeyCode::Char('q') => {
                // Quote selected post
                if let Some(idx) = app.topic_posts_list_state.selected() {
                    if let Some(post) = app.current_topic_posts.get(idx) {
                        if let Some(topic) = app.topics.get(app.selected_topic_idx) {
                            // Construct quote
                            let content = post.raw.as_deref().unwrap_or(&post.cooked);
                            let quote = format!(
                                "[quote=\"{}, post:{}, topic:{}\"]\n{}\n[/quote]\n\n",
                                post.username,
                                post.post_number,
                                topic.id,
                                content
                            );

                            // Pre-fill composer with quote
                            app.topic_composer_input = quote;
                            app.topic_reply_to_post_number = Some(post.post_number);
                            app.topic_visual_mode = false;
                            app.topic_visual_selected_post = None;
                            app.topic_composer_visible = true;
                            app.topic_composer_insert_mode = true;
                        }
                    }
                }
            }
            KeyCode::Char('v') | KeyCode::Esc => {
                // Exit visual mode
                app.topic_visual_mode = false;
                app.topic_visual_selected_post = None;
            }
            _ => {}
        }
    } else {
        // Handle navigation
        match key.code {
            KeyCode::Char('q') => std::process::exit(0),
            KeyCode::Esc => {
                app.goto_screen(AppScreen::MainScreen);
            }
            KeyCode::Char('v') => {
                // Enter visual mode
                app.topic_visual_mode = true;
                app.topic_visual_selected_post = app.topic_posts_list_state.selected();
            }
            KeyCode::Char(' ') => {
                // Show composer for reply
                app.topic_reply_to_post_number = app.topic_posts_list_state.selected()
                    .and_then(|idx| app.current_topic_posts.get(idx))
                    .map(|post| post.post_number);
                app.topic_composer_visible = true;
                app.topic_composer_insert_mode = false; // Start in normal mode
            }
            KeyCode::Char('r') => {
                // Reply to selected post
                app.topic_reply_to_post_number = app.topic_posts_list_state.selected()
                    .and_then(|idx| app.current_topic_posts.get(idx))
                    .map(|post| post.post_number);
                app.topic_composer_visible = true;
                app.topic_composer_insert_mode = false; // Start in normal mode
            }
            KeyCode::Char('R') => {
                // Reply to topic (post #1)
                app.topic_reply_to_post_number = Some(1);
                app.topic_composer_visible = true;
                app.topic_composer_insert_mode = false; // Start in normal mode
            }
            KeyCode::Enter => {
                // Open full post view
                app.post_scroll_offset = 0;
                app.goto_screen(AppScreen::PostView);
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
    }
    Ok(())
}

fn handle_post_view_input(key: event::KeyEvent, app: &mut App) -> io::Result<()> {
    match key.code {
        KeyCode::Char('q') => std::process::exit(0),
        KeyCode::Esc => {
            app.goto_screen(AppScreen::TopicView);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.post_scroll_offset = app.post_scroll_offset.saturating_add(1);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.post_scroll_offset = app.post_scroll_offset.saturating_sub(1);
        }
        _ => {}
    }
    Ok(())
}

async fn apply_filter(app: &mut App, filter: ViewFilter) -> Result<(), Box<dyn std::error::Error>> {
    app.current_filter = filter;

    match filter {
        ViewFilter::AllTopics => {
            app.topics = app.all_topics.clone();
        }
        ViewFilter::MyPosts => {
            app.topics = app
                .all_topics
                .iter()
                .filter(|t| t.author == "ducks")
                .cloned()
                .collect();
        }
        ViewFilter::MyMessages => {
            app.topics = vec![];
        }
        ViewFilter::Category(cat_idx) => {
            // Fetch topics for this category from API
            if let Some(category) = app.categories.get(cat_idx).cloned() {
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

                let category_response = client.get_category_topics(category.id).await?;

                // Convert API topics to TUI topics
                app.topics = category_response
                    .topic_list
                    .topics
                    .iter()
                    .map(|api_topic| {
                        let category_name = api_topic
                            .category_id
                            .and_then(|cat_id| {
                                app.categories
                                    .iter()
                                    .find(|c| c.id == cat_id)
                                    .map(|c| c.name.clone())
                            })
                            .unwrap_or_else(|| "uncategorized".to_string());

                        let author = api_topic
                            .posters
                            .first()
                            .and_then(|p| {
                                category_response
                                    .users
                                    .iter()
                                    .find(|u| u.id as i64 == p.user_id)
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
            } else {
                app.topics = vec![];
            }
        }
        ViewFilter::Tag(_) => {
            app.topics = app.all_topics.clone();
        }
    }

    if !app.topics.is_empty() {
        app.topics_state.select(Some(0));
    }

    Ok(())
}

async fn handle_chat_channels_input(key: event::KeyEvent, app: &mut App) -> io::Result<()> {
    match key.code {
        KeyCode::Char('q') => std::process::exit(0),
        KeyCode::Esc => {
            app.goto_screen(AppScreen::MainScreen);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let len = app.chat_channels.len();
            if len > 0 {
                let i = app.chat_channels_list_state.selected().unwrap_or(0);
                app.chat_channels_list_state.select(Some(if i >= len - 1 { 0 } else { i + 1 }));
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let len = app.chat_channels.len();
            if len > 0 {
                let i = app.chat_channels_list_state.selected().unwrap_or(0);
                app.chat_channels_list_state.select(Some(if i == 0 { len - 1 } else { i - 1 }));
            }
        }
        KeyCode::Enter => {
            if let Some(idx) = app.chat_channels_list_state.selected() {
                if let Some(channel) = app.chat_channels.get(idx) {
                    let channel_id = channel.id;
                    if let Err(e) = load_chat_messages(app, channel_id).await {
                        eprintln!("Failed to load chat messages: {}", e);
                    } else {
                        app.goto_screen(AppScreen::ChatMessages);
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_chat_messages_input(key: event::KeyEvent, app: &mut App) -> io::Result<()> {
    if app.chat_composer_visible && app.chat_composer_insert_mode {
        // INSERT MODE: Can type
        match key.code {
            KeyCode::Esc => {
                // Exit insert mode, back to normal mode
                app.chat_composer_insert_mode = false;
            }
            KeyCode::Enter => {
                // Enter adds newline in insert mode
                app.chat_composer_input.push('\n');
            }
            KeyCode::Backspace => {
                app.chat_composer_input.pop();
            }
            KeyCode::Char(c) => {
                app.chat_composer_input.push(c);
            }
            _ => {}
        }
    } else if app.chat_composer_visible {
        // NORMAL MODE: Composer visible but can't type
        match key.code {
            KeyCode::Char('i') => {
                // Enter insert mode
                app.chat_composer_insert_mode = true;
            }
            KeyCode::Enter => {
                // Send message in normal mode
                if !app.chat_composer_input.trim().is_empty() {
                    if let Some(channel_id) = app.selected_channel_id {
                        let message = app.chat_composer_input.clone();
                        if let Err(e) = send_chat_message(app, channel_id, &message).await {
                            eprintln!("Failed to send message: {}", e);
                        } else {
                            app.chat_composer_input.clear();
                            app.chat_composer_visible = false;
                            app.chat_composer_insert_mode = false;
                            // Refresh messages to show our new message
                            let _ = load_chat_messages(app, channel_id).await;
                        }
                    }
                }
            }
            KeyCode::Esc => {
                // Hide composer
                app.chat_composer_visible = false;
                app.chat_composer_insert_mode = false;
            }
            _ => {}
        }
    } else {
        // Handle navigation
        match key.code {
            KeyCode::Char('q') => std::process::exit(0),
            KeyCode::Esc => {
                app.goto_screen(AppScreen::ChatChannels);
            }
            KeyCode::Char('i') | KeyCode::Char(' ') => {
                // Show composer
                app.chat_composer_visible = true;
                app.chat_composer_insert_mode = false; // Start in normal mode
            }
            KeyCode::Char('r') => {
                // Refresh messages
                if let Some(channel_id) = app.selected_channel_id {
                    if let Err(e) = load_chat_messages(app, channel_id).await {
                        eprintln!("Failed to refresh messages: {}", e);
                    }
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let len = app.current_chat_messages.len();
                if len > 0 {
                    let i = app.chat_messages_list_state.selected().unwrap_or(0);
                    app.chat_messages_list_state.select(Some(if i >= len - 1 { 0 } else { i + 1 }));
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let len = app.current_chat_messages.len();
                if len > 0 {
                    let i = app.chat_messages_list_state.selected().unwrap_or(0);
                    app.chat_messages_list_state.select(Some(if i == 0 { len - 1 } else { i - 1 }));
                }
            }
            _ => {}
        }
    }
    Ok(())
}
