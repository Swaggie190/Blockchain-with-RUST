# Blockchain Miner Project

A simple blockchain implementation with mining capabilities, built in Rust.

## Overview

This project implements a basic blockchain system with proof-of-work mining. It consists of two main components:

1. **Miner**: A command-line tool for mining new blocks and interacting with the blockchain.
2. **Server**: A web server that maintains the blockchain and handles block submissions.

## Features

- Tree-based blockchain structure to handle forks
- Proof-of-work mining with configurable difficulty
- Command-line interface for mining and viewing the blockchain
- Network communication between miners and the server

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version recommended)
- An internet connection for miners to communicate with the server

### Installation

1. Clone the repository:
   ```
   git clone <repository-url>
   cd project2025
   ```

2. Build the project:
   ```
   cargo build
   ```

## Usage

### Starting the Server

Run the server to maintain the blockchain and accept new blocks:

```
cargo run --package server
```

The server will start on `http://localhost:8080` by default.

### Mining Blocks

Use the miner to create new blocks:

```
cargo run --package miner -- mine -d <difficulty> -m <miner-name> --max-iter <iterations>
```

Parameters:
- `-d, --difficulty`: The mining difficulty (default: 5)
- `-m, --miner-name`: Your unique miner name (default: "changemeyoufool")
- `--max-iter`: Maximum number of blocks to mine (optional)

Example:
```
cargo run --package miner -- mine -d 3 -m "my_miner" --max-iter 10
```

### Viewing the Blockchain

To view the current state of the blockchain:

```
cargo run --package miner -- print -d <difficulty>
```

## Project Structure

- `miner/`: Contains the miner implementation
  - `src/block.rs`: Block structure and proof-of-work implementation
  - `src/miner.rs`: Main mining logic and CLI
  - `src/simpletree.rs`: Tree structure for the blockchain
  - `src/network.rs`: Network communication with the server
- `server/`: Contains the blockchain server implementation

## Technical Details

### Block Structure

Each block contains:
- Parent hash: Hash of the parent block
- Miner name: Identifier of the miner who created the block
- Nonce: Value used for proof-of-work
- Dance move: An arbitrary value that affects the block hash

### Mining Algorithm

The mining process follows these steps:
1. Get the latest blocks from the server
2. Select a parent block
3. Create a new block with a random nonce and dance move
4. Find a valid nonce that satisfies the difficulty requirement
5. Submit the block to the server

### Blockchain Structure

The blockchain is represented as a tree structure, where:
- Each node is a block
- Each block can have multiple children
- The longest valid chain is considered the main chain

## License

[MIT License](LICENSE)

## Acknowledgments

- This project was developed as part of a blockchain learning exercise
- Thanks to all contributors who helped improve the codebase
