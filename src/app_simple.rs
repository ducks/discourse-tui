use crate::config::Config;
use discourse_api_rs::{Category, ChatChannel, ChatMessage, Notification, Post};
use ratatui::widgets::ListState;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppScreen {
    ForumPicker,
    MainScreen,
    TopicView,
    PostView,
    ChatChannels,
    ChatMessages,
    Notifications,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaneFocus {
    Sidebar,
    Topics,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewFilter {
    AllTopics,
    MyPosts,
    MyMessages,
    Category(usize),
    Tag(usize),
}

pub struct SidebarItem {
    pub label: String,
    pub unread_count: Option<usize>,
    pub color: Option<ratatui::style::Color>,
    pub filter: Option<ViewFilter>,
}

#[derive(Clone)]
pub struct Topic {
    pub id: usize,
    pub title: String,
    pub author: String,
    pub category: String,
    pub replies: usize,
    pub unread: bool,
}

pub struct App {
    pub screen: AppScreen,
    pub config: Config,

    // Forum picker state
    pub forum_picker_list_state: ListState,
    pub forum_picker_mode: ForumPickerMode,
    pub add_forum_state: AddForumState,

    // Main screen state
    pub focus: PaneFocus,
    pub sidebar_state: ListState,
    pub topics_state: ListState,
    pub sidebar_items: Vec<SidebarItem>,
    pub current_filter: ViewFilter,
    pub current_page: u32,

    // Data
    pub topics: Vec<Topic>,
    pub all_topics: Vec<Topic>,
    pub categories: Vec<Category>,

    // Topic view state
    pub selected_topic_idx: usize,
    pub current_topic_posts: Vec<Post>,
    pub current_topic_id: Option<u64>,
    pub current_topic_all_post_ids: Vec<u64>,
    pub current_topic_view_start: usize, // Which index in the stream we're viewing from
    pub topic_posts_list_state: ListState,
    pub topic_composer_input: String,
    pub topic_composer_visible: bool,
    pub topic_composer_insert_mode: bool,
    pub topic_reply_to_post_number: Option<u32>,
    pub topic_visual_mode: bool,
    pub topic_visual_selected_post: Option<usize>,

    // Post view state
    pub post_scroll_offset: usize,
    pub post_image_urls: HashMap<u64, Vec<String>>,

    // Chat state
    pub chat_channels: Vec<ChatChannel>,
    pub chat_channels_list_state: ListState,
    pub current_chat_messages: Vec<ChatMessage>,
    pub chat_messages_list_state: ListState,
    pub selected_channel_id: Option<u64>,
    pub last_message_poll: std::time::Instant,
    pub chat_composer_input: String,
    pub chat_composer_visible: bool,
    pub chat_composer_insert_mode: bool,

    // Notifications state
    pub notifications: Vec<Notification>,
    pub notifications_list_state: ListState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ForumPickerMode {
    List,
    AddForum,
}

#[derive(Debug, Clone, Default)]
pub struct AddForumState {
    pub name: String,
    pub url: String,
    pub user_api_key: String,
    pub api_key: String,
    pub username: String,
    pub active_field: usize,
    pub error_message: Option<String>,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::load()?;

        let screen = if config.forums.is_empty() {
            AppScreen::ForumPicker
        } else if config.current.selected.is_none() {
            AppScreen::ForumPicker
        } else {
            AppScreen::MainScreen
        };

        let forum_picker_mode = if config.forums.is_empty() {
            ForumPickerMode::AddForum
        } else {
            ForumPickerMode::List
        };

        let mut forum_picker_list_state = ListState::default();
        forum_picker_list_state.select(Some(0));

        let mut sidebar_state = ListState::default();
        sidebar_state.select(Some(0));

        let mut topics_state = ListState::default();
        topics_state.select(Some(0));

        let mut topic_posts_list_state = ListState::default();
        topic_posts_list_state.select(Some(0));

        let mut chat_channels_list_state = ListState::default();
        chat_channels_list_state.select(Some(0));

        let mut chat_messages_list_state = ListState::default();
        chat_messages_list_state.select(Some(0));

        let mut notifications_list_state = ListState::default();
        notifications_list_state.select(Some(0));

        Ok(Self {
            screen,
            config,
            forum_picker_list_state,
            forum_picker_mode,
            add_forum_state: AddForumState::default(),
            focus: PaneFocus::Sidebar,
            sidebar_state,
            topics_state,
            sidebar_items: vec![],
            current_filter: ViewFilter::AllTopics,
            current_page: 0,
            topics: vec![],
            all_topics: vec![],
            categories: vec![],
            selected_topic_idx: 0,
            current_topic_posts: vec![],
            current_topic_id: None,
            current_topic_all_post_ids: vec![],
            current_topic_view_start: 0,
            topic_posts_list_state,
            topic_composer_input: String::new(),
            topic_composer_visible: false,
            topic_composer_insert_mode: false,
            topic_reply_to_post_number: None,
            topic_visual_mode: false,
            topic_visual_selected_post: None,
            post_scroll_offset: 0,
            post_image_urls: HashMap::new(),
            chat_channels: vec![],
            chat_channels_list_state,
            current_chat_messages: vec![],
            chat_messages_list_state,
            selected_channel_id: None,
            last_message_poll: std::time::Instant::now(),
            chat_composer_input: String::new(),
            chat_composer_visible: false,
            chat_composer_insert_mode: false,
            notifications: vec![],
            notifications_list_state,
        })
    }

    pub fn goto_screen(&mut self, screen: AppScreen) {
        self.screen = screen;
    }
}
