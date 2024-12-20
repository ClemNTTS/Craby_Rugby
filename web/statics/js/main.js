async function loadConfig() {
  try {
    const response = await fetch("/config.json");
    return await response.json();
  } catch (error) {
    console.error("Erreur de chargement de la configuration:", error);
    return {
      host: "127.0.0.1",
      port: 8080,
    };
  }
}

class GameClient {
  constructor() {
    this.socket = null;
    this.config = null;
    this.players = {};
    this.my_id = 0;
    this.stamina = 0;
    this.rows = 10;
    this.cols = 10;
    this.stamina_bar = document.getElementById("stamina_bar");
    this.initialize();
  }

  async initialize() {
    try {
      this.config = await loadConfig();
      this.setupWebSocket();
      this.createGameBoard(this.rows, this.cols);
      this.setupEventListeners();
    } catch (error) {
      console.error("Erreur d'initialisation:", error);
    }
  }

  setupWebSocket() {
    this.socket = new WebSocket(`ws://${this.config.host}:${this.config.port}`);

    this.socket.onopen = () => {
      console.log("Connected to WebSocket server");
      document.getElementById("status").textContent = "Connected";
    };

    this.socket.onclose = () => {
      console.log("Connection closed");
      document.getElementById("status").textContent = "Disconnected";
    };

    this.socket.onmessage = (event) => {
      const data = JSON.parse(event.data);
      console.log("Received message:", data);
      this.update_stamina_bar(this.stamina, 100);

      if (data.players) {
        console.log("Updating players with:", data.players);
        this.updatePlayers(data.players);
        this.updateFlag(data.players, data.flag_position);
      } else if (data.player_id) {
        console.log("Found id:", data.player_id);
        this.my_id = data.player_id;
        console.log("Found stamina:", data.stamina);
        this.stamina = data.stamina;
      }
    };
  }

  createGameBoard(rows, cols) {
    const board = document.getElementById("game-board");
    board.style.gridTemplateRows = `repeat(${rows}, 1fr)`;
    board.style.gridTemplateColumns = `repeat(${cols}, 1fr)`;

    for (let i = 0; i < rows; i++) {
      for (let j = 0; j < cols; j++) {
        const cell = document.createElement("div");
        cell.classList.add("cell");
        cell.id = `cell-${i}-${j}`;

        if (i === 0) {
          cell.classList.add("red_goal");
        } else if (i === rows - 1) {
          cell.classList.add("blue_goal");
        }

        board.appendChild(cell);
      }
    }
  }

  updateFlag(players, defaultFlagPosition) {
    document.querySelectorAll(".cell").forEach((cell) => {
      cell.classList.remove("flag");
      if (cell.textContent.includes("🦀")) {
        cell.textContent = cell.textContent.replace("🦀", "");
      }
    });

    let flagHolder = null;
    for (const player of Object.values(players)) {
      if (player.has_flag) {
        flagHolder = player;
        break;
      }
    }

    if (flagHolder) {
      const cell = document.getElementById(
        `cell-${flagHolder.position[1]}-${flagHolder.position[0]}`
      );
      if (cell && cell.textContent) {
        cell.textContent += "🦀";
      }
    } else if (defaultFlagPosition) {
      const cell = document.getElementById(
        `cell-${defaultFlagPosition[1]}-${defaultFlagPosition[0]}`
      );
      if (cell && !cell.classList.contains("player")) {
        cell.classList.add("flag");
        cell.textContent = "🦀";
      }
    }
  }

  updatePlayers(playerList) {
    console.log("Updating players:", playerList);
    this.clearBoard();
    Object.values(playerList).forEach((player) => this.move_player(player));
  }

  move_player(player) {
    const id = player.id;
    const position = player.position;

    if (id === this.my_id) {
      this.stamina = player.stamina;
      this.stamina_bar.textContent = String(this.stamina);
    }

    console.log(`Updating player ${id} at position ${position}`);
    this.players[id] = { x: position[0], y: position[1] };

    const cell = document.getElementById(`cell-${position[1]}-${position[0]}`);

    if (cell.classList.contains("flag") && id === this.my_id) {
      this.socket.send(
        JSON.stringify({
          player_id: this.my_id,
          x: position[0],
          y: position[1],
        })
      );
    }

    if (cell) {
      console.log(`Found cell for player ${id}`);
      if (id % 2 === 0) {
        cell.classList.add("odd_player");
      } else {
        cell.classList.add("even_player");
      }
      let playerText = `P${id}`;
      if (player.has_flag) {
        playerText += "🦀";
        cell.classList.add("player-has-flag");
      } else {
        cell.classList.remove("player-has-flag");
      }
      cell.textContent = playerText;
    }
  }

  clearBoard() {
    document.querySelectorAll(".cell").forEach((cell) => {
      cell.classList.remove("odd_player", "even_player", "player-has-flag");
      cell.textContent = "";
    });
  }

  update_stamina_bar(currentValue, maxValue) {
    const percentage = (currentValue / maxValue) * 100;
    this.stamina_bar.style.width = `${percentage}%`;

    this.stamina_bar.classList.remove("low", "critical");
    if (percentage < 25) {
      this.stamina_bar.classList.add("critical");
    } else if (percentage < 50) {
      this.stamina_bar.classList.add("low");
    }
  }

  setupEventListeners() {
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
      this.socket.send(JSON.stringify({ action: "move", direction }));
    });
  }
}

// Démarrer le jeu
const game = new GameClient();
