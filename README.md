# Chat

## About

Chat is a websocket based chat. (Written in rust btw)

There are 2 main parts in this repo:

- Server (located in the `chat_server` directory)
- Client (located in the `chat_client` directory)

## Server

The server is written using `axum` (previously `rocket`), which uses `tokio-tungstenite` for websockets under the hood.

## Client

The client is a tui application written using `ratatui` for terminal drawing and `tokio-tungstenite` for websocket handling.

## Usage

### Server

As of writing, the server doesn't support args or configs.

```sh
cargo run --bin chat_server
```

### Client

Launch the tui and join the default room:

```sh
cargo run --bin chat_client
```

> [!NOTE]
> The wollowing code snippets will simplify the cargo run bit to just `chat_client`

Launch the tui and join the room called `not_default`:

```sh
chat_client -r not_default
```

Delete the log file and then launch the tui:

```sh
chat_client -c
```

Check the server status:

```sh
chat_client ls
```

Check the server status and display the users currently in the default room:

```sh
chat_client ls -u
```

Or

```sh
chat_client ls --users
```

Check the server and display the users in the room called `something`:

```sh
chat_client ls -ur something
```

## Prerequisites

- [Rust toolchain](https://rust-lang.org/tools/install/)
