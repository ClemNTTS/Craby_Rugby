mod game;
mod player;

mod config;
use config::GameConfig;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

use game::{recharge_stamina, GameState};
use player::Player;

#[derive(Serialize, Deserialize)]
struct ClientAction {
    action: String,
    direction: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct FlagPosition {
    player_id: i32,
    x: i32,
    y: i32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = GameConfig::load();
    println!("Game Config: {:?}", config.max_stamina);
    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Server running on {}", addr);

    let game_state = Arc::new(Mutex::new(GameState::new()));
    let clients: Arc<Mutex<HashMap<u8, tokio::sync::mpsc::UnboundedSender<Message>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let mut player_id_counter = 0;

    let game_state_guard = Arc::clone(&game_state);
    tokio::spawn(recharge_stamina(game_state_guard));

    while let Ok((stream, _)) = listener.accept().await {
        let game_state_clone = Arc::clone(&game_state);
        let clients = Arc::clone(&clients);
        player_id_counter += 1;

        tokio::spawn(handle_connection(
            stream,
            game_state_clone,
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

    let initial_message = InitialMessage {
        player_id: player_id,
        stamina: player.stamina,
    };

    // Send the initial message
    sender
        .send(Message::Text(
            serde_json::to_string(&initial_message).unwrap(),
        ))
        .await
        .unwrap();

    // Envoyer l'état initial
    sender
        .send(Message::Text(
            serde_json::to_string(&*game_state.lock().await).unwrap(),
        ))
        .await
        .unwrap();

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            sender.send(msg).await.unwrap();
        }
    });

    // Periodically send the game state to all connected clients
    let game_state_clone = Arc::clone(&game_state);
    let clients_clone = Arc::clone(&clients);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            let state = serde_json::to_string(&*game_state_clone.lock().await).unwrap();
            for (_, tx) in clients_clone.lock().await.iter() {
                let _ = tx.send(Message::Text(state.clone()));
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        if let Ok(action) = serde_json::from_str::<ClientAction>(msg.to_text().unwrap_or("")) {
            let mut game = game_state.lock().await;
            if action.action == "move" {
                if let Some(direction) = action.direction {
                    let update_needed = game.handle_player_movement(player_id, &direction);
                    if update_needed {
                        // Diffuser game state
                        let state = serde_json::to_string(&*game).unwrap();
                        for (_, tx) in clients.lock().await.iter() {
                            let _ = tx.send(Message::Text(state.clone()));
                        }
                    }
                }
            }
        } else if let Ok(flag) = serde_json::from_str::<FlagPosition>(msg.to_text().unwrap_or("")) {
            let mut game = game_state.lock().await;
            let mut flag_updated = false;
            if flag.player_id != 0 {
                for (_, player) in game.players.iter_mut() {
                    if player.id == flag.player_id as u8 {
                        player.has_flag = true;
                        flag_updated = true;
                        break;
                    }
                }
            }

            if flag_updated {
                let state = serde_json::to_string(&*game).unwrap();
                drop(game);

                let clients_lock = clients.lock().await;
                for (_, tx) in clients_lock.iter() {
                    let _ = tx.send(Message::Text(state.clone()));
                }
            }

            {
                let mut game = game_state.lock().await;
                for player in game.players.iter_mut() {
                    if player.1.has_flag {
                        game.flag_position = player.1.position;
                        break;
                    }
                }
            }
            let state = serde_json::to_string(&*game_state.lock().await).unwrap();
            let clients_lock = clients.lock().await;
            for (_, tx) in clients_lock.iter() {
                let _ = tx.send(Message::Text(state.clone()));
            }
        }
    }

    println!("Player {} disconnected", player_id);
    clients.lock().await.remove(&player_id);
    game_state.lock().await.players.remove(&player_id);
}

#[derive(Serialize, Deserialize)]
struct InitialMessage {
    player_id: u8,
    stamina: i32,
}
