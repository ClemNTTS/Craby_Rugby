use futures_util::SinkExt;
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;

pub const GRID_WIDTH: i32 = 10;
pub const GRID_HEIGHT: i32 = 10;
pub const STAMINA_COST: u8 = 10;

pub struct Player {
    id: u8,
    name: String,
    position: (i32, i32),
    stamina: i32,
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

async fn handle_movement(
    ws_stream: &mut WebSocketStream<TcpStream>,
    player: &mut Player,
    direction: &str,
) {
    const STAMINA_COST: u8 = 10;

    if player.stamina < STAMINA_COST as i32 {
        let response = json!({error: "Not enough stamina"});
        ws_stream.send(response.to_string().into().await.unwrap());
        return;
    }

    match direction {
        "up" if player.position.1 > 0 => player.position.1 -= 1,
        "down" if player.position.1 < GRID_HEIGHT => player.position.1 += 1,
        "left" if player.position.0 > 0 => player.position.0 -= 1,
        "right" if player.position.0 < GRID_WIDTH => player.position.0 += 1,
        _ => {
            let response = json!({error: "Invalid direction"});
            ws_stream.send(response.to_string().into().await.unwrap());
            return;
        }
    }

    player.stamina -= STAMINA_COST as i32;
    let response = json!({
        id: player.id,
        position: player.position,
        stamina: player.stamina,
    });
    ws_stream.send_all(response.to_string().into().await.unwrap());
}

pub async fn recharge_stamina(players: &mut HashMap<String, Player>) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_sec(1)).await;
        for player in players.values_mut() {
            player.stamina = std::cmp::min(player.stamina + 1, MAX_STAMINA);
        }
    }
}
