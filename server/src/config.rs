use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
pub struct GameConfig {
    pub host: String,
    pub port: u16,
    pub grid_width: i32,
    pub grid_height: i32,
    pub initial_flag_position: (i32, i32),
    pub stamina_cost: i32,
    pub max_stamina: i32,
    pub stamina_recharge_rate: i32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            grid_width: 10,
            grid_height: 10,
            initial_flag_position: (4, 4),
            stamina_cost: 10,
            max_stamina: 100,
            stamina_recharge_rate: 10,
        }
    }
}

impl GameConfig {
    pub fn load() -> Self {
        match fs::read_to_string("config.json") {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => {
                let config = Self::default();
                let _ = fs::write(
                    "config.json",
                    serde_json::to_string_pretty(&config).unwrap(),
                );
                config
            }
        }
    }
}
