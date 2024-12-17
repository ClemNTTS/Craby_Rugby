use crate::player::Player;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;

pub const FLAG_POSITION: (i32, i32) = (5, 5);
pub const MAX_STAMINA: i32 = 100;

#[derive(Serialize)]
pub struct GameState {
    pub players: HashMap<u8, Player>,
    pub flag_position: (i32, i32),
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            flag_position: FLAG_POSITION,
        }
    }

    pub fn update_player(&mut self, player: &Player) {
        self.players.insert(player.id, player.clone());
    }
}

pub async fn broadcast_state(
    game_state: Arc<Mutex<GameState>>,
    sender: &mut tokio_tungstenite::tungstenite::protocol::Message,
) -> String {
    let state = game_state.lock().await;
    serde_json::to_string(&*state).unwrap()
}

pub async fn recharge_stamina(players: Arc<tokio::sync::Mutex<GameState>>) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let mut players_guard = players.lock().await;

        for player in players_guard.players.values_mut() {
            player.stamina = std::cmp::min(player.stamina + 5, MAX_STAMINA); // Recharge de 5 par seconde
        }
    }
}
