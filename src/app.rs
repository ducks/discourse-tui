use crate::config::{Config, Forum};
use discourse_api_rs::{Category, LatestResponse};
use ratatui::widgets::ListState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaneFocus {
    Sidebar,
    Topics,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    ForumPicker,
    AddForum,
    TopicList,
    TopicView,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewFilter {
    AllTopics,
    MyPosts,
    MyMessages,
    Category(usize), // Index into categories array
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
    pub screen: Screen,
    pub focus: PaneFocus,
    pub sidebar_state: ListState,
    pub topics_state: ListState,
    pub sidebar_items: Vec<SidebarItem>,
    pub topics: Vec<Topic>,
    pub current_filter: ViewFilter,
    pub all_topics: Vec<Topic>,
    pub categories: Vec<Category>,
    pub config: Config,
    pub forum_picker_state: ListState,
    pub add_forum_inputs: AddForumInputs,
}

#[derive(Debug, Clone, Default)]
pub struct AddForumInputs {
    pub name: String,
    pub url: String,
    pub api_key: String,
    pub username: String,
    pub active_field: usize, // 0=name, 1=url, 2=api_key, 3=username
}

impl App {
    pub fn from_config() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::load()?;

        let mut forum_picker_state = ListState::default();
        forum_picker_state.select(Some(0));

        // Determine starting screen
        let screen = if config.forums.is_empty() {
            Screen::AddForum
        } else if config.current.selected.is_none() {
            Screen::ForumPicker
        } else {
            Screen::TopicList
        };

        Ok(Self {
            screen,
            focus: PaneFocus::Sidebar,
            sidebar_state: ListState::default(),
            topics_state: ListState::default(),
            sidebar_items: vec![],
            topics: vec![],
            current_filter: ViewFilter::AllTopics,
            all_topics: vec![],
            categories: vec![],
            config,
            forum_picker_state,
            add_forum_inputs: AddForumInputs::default(),
        })
    }

    pub fn new() -> Self {
        let mut sidebar_state = ListState::default();
        sidebar_state.select(Some(0));

        let mut topics_state = ListState::default();
        topics_state.select(Some(0));

        let all_topics = Self::create_mock_topics();

        Self {
            screen: Screen::TopicList,
            focus: PaneFocus::Sidebar,
            sidebar_state,
            topics_state,
            sidebar_items: Self::create_sidebar_items(),
            topics: all_topics.clone(),
            current_filter: ViewFilter::AllTopics,
            all_topics,
            categories: vec![],
            config: Config::default(),
            forum_picker_state: ListState::default(),
            add_forum_inputs: AddForumInputs::default(),
        }
    }

    pub fn with_data(latest: LatestResponse, categories: Vec<Category>) -> Self {
        let mut sidebar_state = ListState::default();
        sidebar_state.select(Some(0));

        let mut topics_state = ListState::default();
        topics_state.select(Some(0));

        // Convert API topics to TUI topics
        let all_topics: Vec<Topic> = latest
            .topic_list
            .topics
            .iter()
            .map(|api_topic| {
                // Find category name from category_id
                let category_name = api_topic
                    .category_id
                    .and_then(|cat_id| {
                        categories
                            .iter()
                            .find(|c| c.id == cat_id)
                            .map(|c| c.name.clone())
                    })
                    .unwrap_or_else(|| "uncategorized".to_string());

                // Get first poster as author
                let author = api_topic
                    .posters
                    .first()
                    .and_then(|p| {
                        latest.users.iter().find(|u| u.id == p.user_id).map(|u| u.username.clone())
                    })
                    .unwrap_or_else(|| "unknown".to_string());

                Topic {
                    id: api_topic.id as usize,
                    title: api_topic.title.clone(),
                    author,
                    category: category_name,
                    replies: api_topic.reply_count as usize,
                    unread: false, // We don't have unread info from public API
                }
            })
            .collect();

        // Build sidebar with real categories
        let sidebar_items = Self::create_sidebar_with_categories(&categories);

        let mut forum_picker_state = ListState::default();
        forum_picker_state.select(Some(0));

        Self {
            screen: Screen::TopicList,
            focus: PaneFocus::Sidebar,
            sidebar_state,
            topics_state,
            sidebar_items,
            topics: all_topics.clone(),
            current_filter: ViewFilter::AllTopics,
            all_topics,
            categories,
            config: Config::default(),
            forum_picker_state,
            add_forum_inputs: AddForumInputs::default(),
        }
    }

    fn create_sidebar_with_categories(categories: &[Category]) -> Vec<SidebarItem> {
        let mut items = vec![
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

        // Add real categories
        for (idx, category) in categories.iter().take(10).enumerate() {
            // Parse hex color from category
            let color = Self::parse_color(&category.color);

            items.push(SidebarItem {
                label: format!("■ {}", category.name),
                unread_count: None,
                color,
                filter: Some(ViewFilter::Category(idx)),
            });
        }

        items.push(SidebarItem {
            label: "All categories".to_string(),
            unread_count: None,
            color: None,
            filter: Some(ViewFilter::AllTopics),
        });

        items
    }

    fn parse_color(hex: &str) -> Option<ratatui::style::Color> {
        // Parse 6-digit hex color (without #)
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(ratatui::style::Color::Rgb(r, g, b))
        } else {
            None
        }
    }

    fn create_sidebar_items() -> Vec<SidebarItem> {
        vec![
            SidebarItem {
                label: "Topics".to_string(),
                unread_count: Some(3),
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
                unread_count: Some(1),
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
            SidebarItem {
                label: "■ biz".to_string(),
                unread_count: Some(2),
                color: Some(ratatui::style::Color::Green),
                filter: Some(ViewFilter::Category(0)),
            },
            SidebarItem {
                label: "■ staff".to_string(),
                unread_count: Some(1),
                color: Some(ratatui::style::Color::Gray),
                filter: Some(ViewFilter::Category(1)),
            },
            SidebarItem {
                label: "■ dev-ops".to_string(),
                unread_count: None,
                color: Some(ratatui::style::Color::Rgb(255, 165, 0)),
                filter: Some(ViewFilter::Category(2)),
            },
            SidebarItem {
                label: "■ todo".to_string(),
                unread_count: Some(5),
                color: Some(ratatui::style::Color::Magenta),
                filter: Some(ViewFilter::Category(3)),
            },
            SidebarItem {
                label: "All categories".to_string(),
                unread_count: None,
                color: None,
                filter: Some(ViewFilter::AllTopics),
            },
            SidebarItem {
                label: "--- TAGS ---".to_string(),
                unread_count: None,
                color: None,
                filter: None,
            },
            SidebarItem {
                label: "weekly-call".to_string(),
                unread_count: None,
                color: None,
                filter: Some(ViewFilter::Tag(0)),
            },
            SidebarItem {
                label: "All tags".to_string(),
                unread_count: None,
                color: None,
                filter: Some(ViewFilter::AllTopics),
            },
        ]
    }

    fn create_mock_topics() -> Vec<Topic> {
        vec![
            Topic {
                id: 1,
                title: "Welcome to Discourse TUI!".to_string(),
                author: "ducks".to_string(),
                category: "dev-ops".to_string(),
                replies: 5,
                unread: true,
            },
            Topic {
                id: 2,
                title: "How to navigate with vim keybindings".to_string(),
                author: "claude".to_string(),
                category: "biz".to_string(),
                replies: 12,
                unread: true,
            },
            Topic {
                id: 3,
                title: "Implementing ratatui is fun".to_string(),
                author: "ducks".to_string(),
                category: "dev-ops".to_string(),
                replies: 3,
                unread: false,
            },
            Topic {
                id: 4,
                title: "Date-Ver specification discussion".to_string(),
                author: "ducks".to_string(),
                category: "todo".to_string(),
                replies: 8,
                unread: true,
            },
            Topic {
                id: 5,
                title: "yaml-janitor release notes".to_string(),
                author: "ducks".to_string(),
                category: "dev-ops".to_string(),
                replies: 2,
                unread: false,
            },
        ]
    }

    pub fn next(&mut self) {
        let list_state = match self.focus {
            PaneFocus::Sidebar => &mut self.sidebar_state,
            PaneFocus::Topics => &mut self.topics_state,
        };

        let list_len = match self.focus {
            PaneFocus::Sidebar => self.sidebar_items.len(),
            PaneFocus::Topics => self.topics.len(),
        };

        let i = match list_state.selected() {
            Some(i) => {
                if i >= list_len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let list_state = match self.focus {
            PaneFocus::Sidebar => &mut self.sidebar_state,
            PaneFocus::Topics => &mut self.topics_state,
        };

        let list_len = match self.focus {
            PaneFocus::Sidebar => self.sidebar_items.len(),
            PaneFocus::Topics => self.topics.len(),
        };

        let i = match list_state.selected() {
            Some(i) => {
                if i == 0 {
                    list_len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        list_state.select(Some(i));
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            PaneFocus::Sidebar => PaneFocus::Topics,
            PaneFocus::Topics => PaneFocus::Sidebar,
        };
    }

    pub fn select(&mut self) {
        match self.focus {
            PaneFocus::Sidebar => {
                if let Some(idx) = self.sidebar_state.selected() {
                    if let Some(filter) = self.sidebar_items[idx].filter {
                        self.apply_filter(filter);
                    }
                }
            }
            PaneFocus::Topics => {
                if self.screen == Screen::TopicList {
                    self.screen = Screen::TopicView;
                }
            }
        }
    }

    fn apply_filter(&mut self, filter: ViewFilter) {
        self.current_filter = filter;
        self.topics = match filter {
            ViewFilter::AllTopics => self.all_topics.clone(),
            ViewFilter::MyPosts => self
                .all_topics
                .iter()
                .filter(|t| t.author == "ducks")
                .cloned()
                .collect(),
            ViewFilter::MyMessages => {
                // Empty messages for now (requires authentication)
                vec![]
            }
            ViewFilter::Category(cat_idx) => {
                // Get category name from index
                if let Some(category) = self.categories.get(cat_idx) {
                    let category_name = &category.name;
                    self.all_topics
                        .iter()
                        .filter(|t| &t.category == category_name)
                        .cloned()
                        .collect()
                } else {
                    self.all_topics.clone()
                }
            }
            ViewFilter::Tag(_) => {
                // No tag filtering yet
                self.all_topics.clone()
            }
        };

        // Reset selection to first item
        if !self.topics.is_empty() {
            self.topics_state.select(Some(0));
        }
    }

    pub fn go_back(&mut self) {
        match self.screen {
            Screen::TopicView => self.screen = Screen::TopicList,
            Screen::AddForum => {
                self.screen = if self.config.forums.is_empty() {
                    Screen::AddForum // Can't go back if no forums exist
                } else {
                    Screen::ForumPicker
                };
            }
            _ => {}
        }
    }

    pub fn handle_char_input(&mut self, c: char) {
        if self.screen == Screen::AddForum {
            let field = match self.add_forum_inputs.active_field {
                0 => &mut self.add_forum_inputs.name,
                1 => &mut self.add_forum_inputs.url,
                2 => &mut self.add_forum_inputs.api_key,
                3 => &mut self.add_forum_inputs.username,
                _ => return,
            };
            field.push(c);
        } else if self.screen == Screen::ForumPicker && c == 'a' {
            self.screen = Screen::AddForum;
            self.add_forum_inputs = AddForumInputs::default();
        } else if self.screen == Screen::ForumPicker && c == 'd' {
            self.delete_selected_forum();
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.screen == Screen::AddForum {
            let field = match self.add_forum_inputs.active_field {
                0 => &mut self.add_forum_inputs.name,
                1 => &mut self.add_forum_inputs.url,
                2 => &mut self.add_forum_inputs.api_key,
                3 => &mut self.add_forum_inputs.username,
                _ => return,
            };
            field.pop();
        }
    }

    pub fn handle_tab(&mut self) {
        match self.screen {
            Screen::AddForum => {
                self.add_forum_inputs.active_field = (self.add_forum_inputs.active_field + 1) % 4;
            }
            Screen::TopicList => self.toggle_focus(),
            _ => {}
        }
    }

    pub fn handle_enter(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.screen {
            Screen::ForumPicker => {
                if let Some(idx) = self.forum_picker_state.selected() {
                    if let Some(forum) = self.config.forums.get(idx) {
                        self.config.set_current_forum(forum.id.clone());
                        self.config.save()?;
                        self.screen = Screen::TopicList;
                        // Will need to load forum data here
                    }
                }
            }
            Screen::AddForum => {
                if !self.add_forum_inputs.name.is_empty() && !self.add_forum_inputs.url.is_empty() {
                    let id = self.add_forum_inputs.name.to_lowercase().replace(' ', "-");
                    let forum = Forum {
                        id: id.clone(),
                        name: self.add_forum_inputs.name.clone(),
                        url: self.add_forum_inputs.url.clone(),
                        api_key: if self.add_forum_inputs.api_key.is_empty() {
                            None
                        } else {
                            Some(self.add_forum_inputs.api_key.clone())
                        },
                        username: if self.add_forum_inputs.username.is_empty() {
                            None
                        } else {
                            Some(self.add_forum_inputs.username.clone())
                        },
                    };
                    self.config.add_forum(forum);
                    self.config.set_current_forum(id);
                    self.config.save()?;
                    self.screen = Screen::TopicList;
                    // Will need to load forum data here
                }
            }
            Screen::TopicList => self.select(),
            _ => {}
        }
        Ok(())
    }

    fn delete_selected_forum(&mut self) {
        if let Some(idx) = self.forum_picker_state.selected() {
            if let Some(forum) = self.config.forums.get(idx).cloned() {
                self.config.remove_forum(&forum.id);
                let _ = self.config.save();

                // Update selection
                if self.config.forums.is_empty() {
                    self.screen = Screen::AddForum;
                } else if idx >= self.config.forums.len() {
                    self.forum_picker_state.select(Some(self.config.forums.len() - 1));
                }
            }
        }
    }

    pub fn handle_navigation(&mut self, down: bool) {
        match self.screen {
            Screen::ForumPicker => {
                let len = self.config.forums.len();
                if len == 0 {
                    return;
                }
                let i = self.forum_picker_state.selected().unwrap_or(0);
                let new_i = if down {
                    if i >= len - 1 { 0 } else { i + 1 }
                } else {
                    if i == 0 { len - 1 } else { i - 1 }
                };
                self.forum_picker_state.select(Some(new_i));
            }
            Screen::TopicList => {
                if down {
                    self.next();
                } else {
                    self.previous();
                }
            }
            _ => {}
        }
    }
}
