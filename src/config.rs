use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub current: CurrentForum,
    pub forums: Vec<Forum>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentForum {
    pub selected: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forum {
    pub id: String,
    pub name: String,
    pub url: String,
    pub api_key: Option<String>,
    pub username: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(&config_path, contents)?;
        Ok(())
    }

    pub fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let proj_dirs = ProjectDirs::from("", "", "discourse-tui")
            .ok_or("Could not determine config directory")?;
        Ok(proj_dirs.config_dir().join("config.toml"))
    }

    pub fn add_forum(&mut self, forum: Forum) {
        self.forums.push(forum);
    }

    pub fn remove_forum(&mut self, id: &str) {
        self.forums.retain(|f| f.id != id);
        if self.current.selected.as_deref() == Some(id) {
            self.current.selected = None;
        }
    }

    pub fn get_current_forum(&self) -> Option<&Forum> {
        let selected = self.current.selected.as_ref()?;
        self.forums.iter().find(|f| &f.id == selected)
    }

    pub fn set_current_forum(&mut self, id: String) {
        self.current.selected = Some(id);
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            current: CurrentForum { selected: None },
            forums: vec![],
        }
    }
}
