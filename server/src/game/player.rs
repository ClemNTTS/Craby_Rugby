use futures_util::SinkExt;
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::WebSocketStream;

pub const GRID_WIDTH: i32 = 10;
pub const GRID_HEIGHT: i32 = 10;
pub const STAMINA_COST: u8 = 10;
pub const MAX_STAMINA: i32 = 50;

#[derive(Clone)]
pub struct Player {
    pub id: u8,
    pub name: String,
    pub position: (i32, i32),
    pub stamina: i32,
}

impl Player {
    pub fn new(id: u8, username: String, position: (i32, i32), stamina: i32) -> Self {
        Player {
            id: id,
            name: username,
            position,
            stamina,
        }
    }
}

pub struct PlayerConnection {
    player: Player,
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
}

async fn handle_movement(
    ws_stream: &mut WebSocketStream<TcpStream>,
    player: &mut Player,
    direction: &str,
    players: &Arc<Mutex<HashMap<String, PlayerConnection>>>,
) {
    const STAMINA_COST: u8 = 10;

    if player.stamina < STAMINA_COST as i32 {
        let response = json!({"error": "Not enough stamina"});
        ws_stream.send(response.to_string().into()).await.unwrap();
        return;
    }

    match direction {
        "up" if player.position.1 > 0 => player.position.1 -= 1,
        "down" if player.position.1 < GRID_HEIGHT => player.position.1 += 1,
        "left" if player.position.0 > 0 => player.position.0 -= 1,
        "right" if player.position.0 < GRID_WIDTH => player.position.0 += 1,
        _ => {
            let response = json!({"error": "Invalid direction"});
            ws_stream.send(response.to_string().into()).await.unwrap();
            return;
        }
    }

    player.stamina -= STAMINA_COST as i32;

    let response = json!({
        "id": player.id,
        "position": player.position,
        "stamina": player.stamina,
    });
    broadcast_to_all_players(response, players).await;
}

pub async fn recharge_stamina(
    players: Arc<Mutex<HashMap<String, Player>>>,
    conn_tab: &Arc<Mutex<HashMap<String, PlayerConnection>>>,
) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let mut players_guard = players.lock().await;
        for player in players_guard.values_mut() {
            player.stamina = std::cmp::min(player.stamina + 1, MAX_STAMINA);
            let message = json!({
               "type": "stamina_update",
               "id": player.id.clone(),
               "stamina": player.stamina.clone(),
            });

            broadcast_to_all_players(message, conn_tab);
        }
    }
}

async fn broadcast_to_all_players(
    message: serde_json::Value,
    players: &Arc<Mutex<HashMap<String, PlayerConnection>>>,
) {
    let mut players_lock = players.lock().await;
    for (_, player_conn) in players_lock.iter_mut() {
        if let Err(e) = player_conn
            .ws_stream
            .send(serde_json::to_string(&message).unwrap().into())
            .await
        {
            eprintln!(
                "Error sending message to {}: {:?}",
                player_conn.player.id, e
            );
        }
    }
}
