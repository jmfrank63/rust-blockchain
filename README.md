# Simple Blockchain in Rust

This project demonstrates a very basic blockchain implementation in Rust.

## Overview

The blockchain stores a chain of `Human` structs, where each `Human` has a `name` and `age`.
New humans are added by creating `Block` structs containing the human data and the hash of
the previous block.

The blockchain logic is handled by the `BlockChain` struct which stores the genesis block and
validates new blocks before adding them.

## Usage

To run the example:

1. Clone the repository

2. Run `cargo run`

This will:

- Create the genesis block with Adam
- Add a new block with Seth
- Add a block with Enos
- Try to add an invalid block with Lucifer and show error

## Code Structure

- `main.rs` - Runs the blockchain sync example
- `simple.rs` - Contains the main blockchain example logic
- `blockchain.rs` - BlockChain and Block structs

## Blockchain Logic

- Defines utility functions like `hash_to_binary_representation` and `calculate_hash` for generating
hashes. Uses SHA256 for cryptographic hashing.

- `mine_block` function to mine a block by finding a valid nonce. Checks that the hash starts with
 enough leading 0s to satisfy the difficulty target.

- `TimeReference` struct to convert between Rust's `Instant` and `SystemTime` for serialization.

- `SerializableInstant` wrapper to serialize/deserialize Instants. Needed for consensus.

- `Block` struct representing each block in the chain. Contains id, timestamp, previous hash, data
payload, nonce, and block hash.

- `Blockchain` struct to store the chain and validate it. Methods like:

  - `genesis` to create the first block
  - `try_add_block` to add a new block after validation
  - `is_block_valid` to check PoW, hashes, etc
  - `is_chain_valid` to validate a whole chain
  - `choose_chain` to pick the best valid chain

- Serialization functions from serde to allow data structs and blocks to be serialized and hashed.

- Tests for the key functionality.
- Run `cargo test` to run the tests.
