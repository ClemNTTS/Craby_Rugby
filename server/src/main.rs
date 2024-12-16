use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "localhost:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("Server running on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            if let Ok(ws_stream) = accept_async(stream).await {
                println!("New WebSocket connection");
                handle_connection(ws_stream).await;
            }
        });
    }
    Ok(())
}

async fn handle_connection(
    mut ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
) {
    while let Some(msg) = ws_stream.next().await {
        if let Ok(msg) = msg {
            if msg.is_text() {
                let received = msg.into_text().unwrap();
                println!("Received: {}", received);
                ws_stream.send(received.into()).await.unwrap();
            }
        }
    }
}
