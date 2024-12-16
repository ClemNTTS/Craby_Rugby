use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::intrinsics::mir::Len;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
mod game;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    let mut nb_players = 0;
    println!("Server running on {}", addr);

    let players_conn: Arc<Mutex<HashMap<String, game::player::PlayerConnection>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let players: Arc<Mutex<HashMap<String, game::player::Player>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let players_conn_for_stamina = Arc::clone(&players_conn);
    let players_for_stamina = Arc::clone(&players);

    tokio::spawn(async move {
        game::player::recharge_stamina(players_for_stamina, &players_conn_for_stamina).await
    });

    while let Ok((stream, _)) = listener.accept().await {
        let players_clone = Arc::clone(&players);
        tokio::spawn(async move {
            if let Ok(ws_stream) = accept_async(stream).await {
                println!("New WebSocket connection");
                handle_connection(ws_stream, players_clone).await;
            }
        });
    }

    Ok(())
}

//Struct for messages
#[derive(Serialize, Deserialize, Debug)]
pub struct ClientMessage {
    pub action: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerMessage {
    pub message: String,
}

async fn handle_connection(
    mut ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    players: Arc<Mutex<HashMap<String, game::player::PlayerConnection>>>,
    nb_players: i32,
) {
    let (sender, mut receiver) = ws_stream.split();
    let player_id = nb_players + 1;
    let player = game::player::Player {
        name: format!("player{}", player_id),
        id: player_id as u8,
        position: (0, 0),
        stamina: 100,
    };
    let player_conn = game::player::PlayerConnection {
        player: player.clone(),
        ws_stream,
    };

    while let Some(msg) = ws_stream.next().await {
        if let Ok(msg) = msg {
            if msg.is_text() {
                let received = msg.into_text().unwrap();
                println!("Received: {}", received);

                //Parse incoming JSON message
                if let Ok(client_mssg) = serde_json::from_str::<ClientMessage>(&received) {
                    println!("Client action : {}", client_mssg.action);

                    let response = ServerMessage {
                        message: format!("Action : {}", client_mssg.action),
                    };

                    let response_json = serde_json::to_string(&response).unwrap();
                    ws_stream.send(response_json.into()).await.unwrap();
                } else {
                    println!("Invalid JSON format");
                }
            }
        }
    }
}
