mod game;
mod player; // Import du module player.rs // Import du module game.rs

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

use game::{broadcast_state, GameState};
use player::Player;

#[derive(Serialize, Deserialize)]
struct ClientAction {
    action: String,
    direction: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await?;
    println!("Server running on {}", addr);

    let game_state = Arc::new(Mutex::new(GameState::new()));
    let clients: Arc<Mutex<HashMap<u8, tokio::sync::mpsc::UnboundedSender<Message>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let mut player_id_counter = 0;

    while let Ok((stream, _)) = listener.accept().await {
        let game_state = Arc::clone(&game_state);
        let clients = Arc::clone(&clients);
        player_id_counter += 1;

        tokio::spawn(handle_connection(
            stream,
            game_state,
            clients,
            player_id_counter,
        ));
    }

    Ok(())
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    game_state: Arc<Mutex<GameState>>,
    clients: Arc<Mutex<HashMap<u8, tokio::sync::mpsc::UnboundedSender<Message>>>>,
    player_id: u8,
) {
    let ws_stream = accept_async(stream).await.unwrap();
    let (mut sender, mut receiver) = ws_stream.split();

    let player = Player::new(player_id, format!("Player{}", player_id));
    println!("New player connected: {}", player.name);

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    {
        clients.lock().await.insert(player_id, tx);
        game_state.lock().await.update_player(&player);
    }

    // Envoyer l'état initial
    sender
        .send(Message::Text(
            serde_json::to_string(&*game_state.lock().await).unwrap(),
        ))
        .await
        .unwrap();

    let game_state_clone = Arc::clone(&game_state);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            sender.send(msg).await.unwrap();
        }
    });

    // Écouter les actions du joueur
    while let Some(Ok(msg)) = receiver.next().await {
        if let Ok(action) = serde_json::from_str::<ClientAction>(msg.to_text().unwrap_or("")) {
            let mut game = game_state.lock().await;
            if let Some(player) = game.players.get_mut(&player_id) {
                if action.action == "move" {
                    if let Some(direction) = action.direction {
                        let player_updated = player.move_player(&direction);
                        if player_updated {
                            let player_clone = player.clone();
                            game.update_player(&player_clone);
                        }
                    }
                }
            }

            // Diffuser l'état du jeu à tous
            let state = serde_json::to_string(&*game).unwrap();
            for (_, tx) in clients.lock().await.iter() {
                let _ = tx.send(Message::Text(state.clone()));
            }
        }
    }

    println!("Player {} disconnected", player_id);
    clients.lock().await.remove(&player_id);
    game_state.lock().await.players.remove(&player_id);
}
