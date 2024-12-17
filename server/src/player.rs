use serde::Serialize;

pub const STAMINA_COST: i32 = 10;
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
        Self {
            id,
            name,
            position: (0, 0),
            stamina: MAX_STAMINA,
            has_flag: false,
        }
    }

    pub fn move_player(&mut self, direction: &str) -> bool {
        if self.stamina < STAMINA_COST {
            return false; // Pas assez de stamina
        }

        match direction {
            "up" => self.position.1 -= 1,
            "down" => self.position.1 += 1,
            "left" => self.position.0 -= 1,
            "right" => self.position.0 += 1,
            _ => return false,
        }

        self.stamina -= STAMINA_COST;
        true
    }
}
