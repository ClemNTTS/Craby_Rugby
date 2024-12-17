use crate::player::Player;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub const FLAG_POSITION: (i32, i32) = (5, 5);

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
