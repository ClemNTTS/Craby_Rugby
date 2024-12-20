use serde::Serialize;

use crate::config;

pub const MAX_STAMINA: i32 = 100;

#[derive(Clone, Serialize)]
pub struct Player {
    pub id: u8,
    pub name: String,
    pub position: (i32, i32),
    pub stamina: i32,
    pub has_flag: bool,
}

impl Player {
    pub fn new(id: u8, name: String) -> Self {
        let mut pos = (0, 0);

        let width = config::GameConfig::load().grid_width;
        let height = config::GameConfig::load().grid_height;

        if id % 2 == 0 {
            pos = (width / 2 - 1, 0);
        } else {
            pos = (width / 2 - 1, height - 1);
        }

        let max = config::GameConfig::load().max_stamina;

        Self {
            id,
            name,
            position: pos,
            stamina: max,
            has_flag: false,
        }
    }
}
