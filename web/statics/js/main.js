const socket = new WebSocket("ws://127.0.0.1:8080");

const rows = 10;
const cols = 10;
const players = {}; // Stocke les positions des joueurs { id: { x, y } }
my_id = 0;
stamina = 0;
const stamina_bar = document.getElementById("stamina_bar");

socket.onopen = () => {
  console.log("Connected to WebSocket server");
  document.getElementById("status").textContent = "Connected";
};

socket.onclose = () => {
  console.log("Connection closed");
  document.getElementById("status").textContent = "Disconnected";
};

// Crée le plateau de jeu avec des IDs uniques pour chaque cellule
function createGameBoard(rows, cols) {
  const board = document.getElementById("game-board");
  board.style.gridTemplateRows = `repeat(${rows}, 1fr)`;
  board.style.gridTemplateColumns = `repeat(${cols}, 1fr)`;

  for (let i = 0; i < rows; i++) {
    for (let j = 0; j < cols; j++) {
      const cell = document.createElement("div");
      cell.classList.add("cell");
      cell.id = `cell-${i}-${j}`; // ID unique pour chaque cellule
      board.appendChild(cell);
    }
  }
}

socket.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log("Received message:", data);
  update_stamina_bar(stamina, 100);

  if (data.players) {
    console.log("Updating players with:", data.players);
    updatePlayers(data.players);
  } else if (data.player_id) {
    console.log("Found id :", data.player_id);
    my_id = data.player_id;
    console.log("Found stamina :", data.stamina);
    stamina = data.stamina;
  } else {
    console.log("Received unknown message type:", data);
  }
};

function updatePlayers(playerList) {
  console.log("Updating players:", playerList);
  clearBoard();

  Object.values(playerList).forEach(move_player);
}

function move_player(player) {
  const id = player.id;
  const position = player.position;

  if (id === my_id) {
    console.log(`HEY`);
    stamina = player.stamina;
    stamina_bar.textContent = String(stamina);
  }

  console.log(`Updating player ${id} at position ${position}`);
  players[id] = { x: position[0], y: position[1] };

  const cell = document.getElementById(`cell-${position[1]}-${position[0]}`);
  if (cell) {
    console.log(`Found cell for player ${id}`);
    cell.classList.add("player");
    cell.textContent = `P${id}`;
  } else {
    console.log(`Could not find cell for player ${id} at position ${position}`);
  }
}

// Efface tous les joueurs du tableau
function clearBoard() {
  document.querySelectorAll(".cell").forEach((cell) => {
    cell.classList.remove("player");
    cell.textContent = "";
  });
}

createGameBoard(rows, cols);

// Envoie les mouvements en fonction des touches fléchées
document.addEventListener("keydown", (event) => {
  let direction = null;

  switch (event.key) {
    case "ArrowUp":
      direction = "up";
      break;
    case "ArrowDown":
      direction = "down";
      break;
    case "ArrowLeft":
      direction = "left";
      break;
    case "ArrowRight":
      direction = "right";
      break;
    default:
      return;
  }

  console.log("Sending move:", direction);
  socket.send(JSON.stringify({ action: "move", direction }));
});

function update_stamina_bar(currentValue, maxValue) {
  const percentage = (currentValue / maxValue) * 100;
  stamina_bar.style.width = `${percentage}%`;

  stamina_bar.classList.remove("low", "critical");
  if (percentage < 25) {
    stamina_bar.classList.add("critical");
  } else if (percentage < 50) {
    stamina_bar.classList.add("low");
  }
}
