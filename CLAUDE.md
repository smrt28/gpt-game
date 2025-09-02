# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Architecture

This is a "Guess Who" game implementation with a Rust-based architecture consisting of three main components:

- **Server** (`server/`): Axum-based HTTP server that manages game logic and integrates with GPT for AI opponents
- **Frontend** (`frontend/`): Yew-based WebAssembly client for the web interface
- **Shared** (`shared/`): Common data structures and error types used across server and frontend

### Key Components

- **Game Manager** (`server/src/game_manager.rs`): Handles game state, player sessions, and game flow
- **GPT Integration** (`server/src/gpt.rs`): Manages OpenAI API communication for AI game opponents
- **Client Pool** (`server/src/client_pool.rs`): Connection pooling for GPT API clients
- **Token Generation** (`server/src/token_gen.rs`): Generates secure tokens for game sessions

## Development Commands

### Server (Rust)
- **Development build**: `cargo build` (from project root or `server/`)
- **Production build**: `cargo build --release --target x86_64-unknown-linux-musl` 
- **Run server**: `cargo run` (from `server/` directory)

### Frontend (WebAssembly)
- **Development server**: `trunk serve` (from `frontend/` directory, serves on port 8000)
- **Build frontend**: `trunk build` (from `frontend/` directory)

### Workspace
- **Build all components**: `cargo build` (from project root)
- **Test all components**: `cargo test` (from project root)

## Configuration

The server uses TOML configuration (`server/assets/config.toml`):
- GPT API settings (key file path, max clients, instructions file)
- Web server configuration (port, static file paths)
- Debug settings

Game instructions are stored in `server/assets/instructions.txt` and define the AI behavior for the "Guess Who" game.

## Architecture Notes

- The server runs on port 3000 by default, frontend development server on port 8000
- GPT client connections are pooled using a custom `PollableClientFactory` pattern
- Game state is managed server-side with session tokens
- Frontend communicates with server via HTTP API endpoints
- All shared types use Serde for JSON serialization between frontend and backend