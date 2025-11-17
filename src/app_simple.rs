use crate::config::Config;
use discourse_api_rs::{Category, ChatChannel, ChatMessage, Post};
use ratatui::widgets::ListState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppScreen {
    ForumPicker,
    MainScreen,
    TopicView,
    PostView,
    ChatChannels,
    ChatMessages,
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

    // Data
    pub topics: Vec<Topic>,
    pub all_topics: Vec<Topic>,
    pub categories: Vec<Category>,

    // Topic view state
    pub selected_topic_idx: usize,
    pub current_topic_posts: Vec<Post>,
    pub topic_posts_list_state: ListState,

    // Post view state
    pub post_scroll_offset: usize,

    // Chat state
    pub chat_channels: Vec<ChatChannel>,
    pub chat_channels_list_state: ListState,
    pub current_chat_messages: Vec<ChatMessage>,
    pub chat_messages_list_state: ListState,
    pub selected_channel_id: Option<u64>,
    pub last_message_poll: std::time::Instant,
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
    pub api_key: String,
    pub username: String,
    pub active_field: usize,
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
            topics: vec![],
            all_topics: vec![],
            categories: vec![],
            selected_topic_idx: 0,
            current_topic_posts: vec![],
            topic_posts_list_state,
            post_scroll_offset: 0,
            chat_channels: vec![],
            chat_channels_list_state,
            current_chat_messages: vec![],
            chat_messages_list_state,
            selected_channel_id: None,
            last_message_poll: std::time::Instant::now(),
        })
    }

    pub fn goto_screen(&mut self, screen: AppScreen) {
        self.screen = screen;
    }
}
