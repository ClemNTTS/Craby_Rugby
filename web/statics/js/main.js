const socket = new WebSocket("ws://127.0.0.1:8080");

socket.onopen = () => {
  console.log("Connected to WebSocket server");
  document.getElementById("status").textContent = "Connected";

  const message = {
    text: "Hello there",
    action: "up",
  };

  socket.send(JSON.stringify(message));
};

socket.onmessage = (event) => {
  console.log("Received message:", event.data);
};

socket.onclose = () => {
  console.log("Connection closed");
  document.getElementById("status").textContent = "Disconnected";
};

function createGameBoard(rows, cols) {
  const board = document.getElementById("game-board");
  for (let i = 0; i < rows; i++) {
    for (let j = 0; j < cols; j++) {
      const cell = document.createElement("div");
      cell.classList.add("cell");
      board.appendChild(cell);
    }
  }
}

createGameBoard(10, 10);

document.addEventListener("keydown", (event) => {
  switch (event.key) {
    case "ArrowUp":
      socket.send(JSON.stringify({ action: "up" }));
      break;
    case "ArrowDown":
      socket.send(JSON.stringify({ action: "down" }));
      break;
    case "ArrowLeft":
      socket.send(JSON.stringify({ action: "left" }));
      break;
    case "ArrowRight":
      socket.send(JSON.stringify({ action: "right" }));
      break;
    default:
      break;
  }
});
