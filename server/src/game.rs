use crate::config;
use crate::player::Player;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize)]
pub struct GameState {
    pub players: HashMap<u8, Player>,
    pub flag_position: (i32, i32),
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            flag_position: config::GameConfig::load().initial_flag_position,
        }
    }

    pub fn update_player(&mut self, player: &Player) {
        self.players.insert(player.id, player.clone());
    }

    pub fn handle_player_movement(&mut self, player_id: u8, direction: &str) -> bool {
        let mut should_reset = false;
        let mut player_updated = false;
        let mut flag_reset = false;

        // Collect other players' info first
        let other_players_info: Vec<(u8, (i32, i32), bool)> = self
            .players
            .iter()
            .filter(|(&id, _)| id != player_id)
            .map(|(&id, player)| (id, player.position, player.has_flag))
            .collect();

        // Handle player movement
        if let Some(player) = self.players.get_mut(&player_id) {
            let current_pos = player.position;
            let mut new_pos = current_pos;

            match direction {
                "up" if current_pos.1 > 0 => new_pos.1 -= 1,
                "down" if current_pos.1 < config::GameConfig::load().grid_height - 1 => {
                    new_pos.1 += 1
                }
                "left" if current_pos.0 > 0 => new_pos.0 -= 1,
                "right" if current_pos.0 < config::GameConfig::load().grid_width - 1 => {
                    new_pos.0 += 1
                }
                _ => return false,
            }

            // Check collisions
            let mut collision_with_flag_holder = false;
            let mut can_move = true;

            for (_, pos, has_flag) in &other_players_info {
                if *pos == new_pos {
                    if *has_flag {
                        collision_with_flag_holder = true;
                    }
                    can_move = false;
                    break;
                }
            }

            if can_move && player.stamina >= config::GameConfig::load().stamina_cost {
                player.position = new_pos;
                player.stamina -= config::GameConfig::load().stamina_cost;
                player_updated = true;

                // Check if player with flag reached goal - modified for full line scoring
                if player.has_flag {
                    if (player.id % 2 == 0
                        && player.position.1 == config::GameConfig::load().grid_height - 1)
                        || (player.id % 2 == 1 && player.position.1 == 0)
                    {
                        should_reset = true;
                    }
                }
            }

            if collision_with_flag_holder {
                flag_reset = true;
            }
        }

        // Handle resets after movement
        if should_reset {
            self.reset_all_positions();
        } else if flag_reset {
            self.reset_flag();
        }

        player_updated || flag_reset || should_reset
    }

    pub fn reset_flag(&mut self) {
        for player in self.players.values_mut() {
            player.has_flag = false;
        }
        self.flag_position = config::GameConfig::load().initial_flag_position;
    }

    pub fn reset_all_positions(&mut self) {
        for player in self.players.values_mut() {
            // Reset player position based on team (even/odd ID)
            if player.id % 2 == 0 {
                player.position = (config::GameConfig::load().grid_width / 2 - 1, 0);
            } else {
                player.position = (
                    config::GameConfig::load().grid_width / 2 - 1,
                    config::GameConfig::load().grid_height - 1,
                );
            }
            player.has_flag = false;
            player.stamina = crate::player::MAX_STAMINA;
        }
        self.flag_position = config::GameConfig::load().initial_flag_position;
    }
}

pub async fn recharge_stamina(players: Arc<tokio::sync::Mutex<GameState>>) {
    let rate = config::GameConfig::load().stamina_recharge_rate;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let mut players_guard = players.lock().await;
        for player in players_guard.players.values_mut() {
            player.stamina = std::cmp::min(
                player.stamina + rate,
                config::GameConfig::load().max_stamina,
            );
        }
    }
}
