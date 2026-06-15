# Chat

## About

Chat is a websocket based chat. (Written in rust btw)

There are 2 main parts in this repo:

- Server (located in the `chat_server` directory)
- Client (located in the `chat_client` directory)

## Server

The server is written using `axum` (previously rocket), which uses `tokio-tungstenite` for websockets under the hood.

## Client

The client is a tui application written using `ratatui` for terminal drawing and `tokio-tungstenite` for websocket handling.

## Prerequisites

- [Rust toolchain](https://rust-lang.org/tools/install/)
