use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
mod game;

//CONSTANTS
pub const STAMINA_RECHARGE_RATE: i32 = 5;
pub const MAX_STAMINA_COST: i32 = 50;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("Server running on {}", addr);

    let players: HashMap<String, game::player::Player> = HashMap::new();

    tokio::spawn(game::player::recharge_stamina(&mut players.clone()));

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let players_clone = players.clone();
            if let Ok(ws_stream) = accept_async(stream).await {
                println!("New WebSocket connection");
                handle_connection(ws_stream, &mut players_clone).await;
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
) {
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
