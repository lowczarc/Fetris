# 🕹️ FETRIS 🕹️

Fetris is a Rust implementation of the classic Tetris game with multiplayer support. The project includes a game server and client, and an AI player. 

![Tetris gameplay screenshot](https://raw.githubusercontent.com/lowczarc/Fetris/master/assets/fetris.png)

## 🚀 Getting started

To run the game, you'll need to install Rust and Cargo. Once you've done that, you can clone the repository and navigate to the root directory.

### 🎮 Starting the server

To start the server with pools of size 10, run:

```sh
cargo run --bin fetris_server -- -s 10
```

This will start the Tetris game server, which will listen for client connections on port 3001 by default.

### 🎮 Starting the client

To start the client, run:

```sh
cargo run --bin fetris_client <server-address>
```

Replace `<server-address>` with the address of the game server (e.g., `localhost:3001`). Use the arrow keys to move the Tetriminos, and press `Enter` to rotate them.

### 🤖 Starting the AI player

To start the AI player, run:

```sh
cargo run --bin ai-player <server-address>
```

Replace `<server-address>` with the address of the game server (e.g., `localhost:3001`). The AI player will automatically play the game, attempting to stay alive for as long as possible.

## 🕹️ Gameplay

The objective of the game is to clear lines by placing Tetriminos in a grid. When a horizontal line is filled with blocks, it will be cleared, and any blocks above it will fall. The game ends when the blocks reach the top of the grid.

In multiplayer mode, players compete to be the last remaining player in their pool. Clearing lines sends junk lines to the other players.

## 📝 License

This project is licensed under the "I don't care about licenses, do what the hell you want with it" license
