use std::{
    pin::Pin,
    task::{Context, Poll},
};

use axum::extract::ws::{CloseFrame as AxumFrame, Message as AxumMessage, WebSocket};
use futures::{Sink, SinkExt, Stream, StreamExt, stream::FusedStream};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, tungstenite::protocol::frame::coding::CloseCode,
};

pub use tokio_tungstenite::tungstenite::{Error, Message, protocol::CloseFrame};

#[cfg(feature = "mock_ws")]
use crate::ws_mock::MockWebSocket;

#[derive(Debug)]
pub enum WsConnection {
    /// The type `tokio_tungstenite` uses when used as a client
    WebSocketClient(Box<WebSocketStream<MaybeTlsStream<TcpStream>>>),
    /// The type `axum` uses, which doesn't expose the underlying `tokio_tungstenite` `WebSocketStream`
    WebSocketServer(Box<WebSocket>),
    #[cfg(feature = "mock_ws")]
    Mock(Box<MockWebSocket>),
}

fn axum_to_tungstenite(msg: AxumMessage) -> Message {
    match msg {
        AxumMessage::Text(b) => Message::text(b.as_str()),
        AxumMessage::Binary(b) => Message::Binary(b),
        AxumMessage::Ping(b) => Message::Ping(b),
        AxumMessage::Pong(b) => Message::Pong(b),
        AxumMessage::Close(Some(close)) => Message::Close(Some(CloseFrame {
            code: CloseCode::from(close.code),
            reason: close.reason.as_str().into(),
        })),
        AxumMessage::Close(None) => Message::Close(None),
    }
}

fn tungstenite_to_axum(msg: Message) -> AxumMessage {
    match msg {
        Message::Text(b) => AxumMessage::text(b.as_str()),
        Message::Binary(b) => AxumMessage::Binary(b),
        Message::Ping(b) => AxumMessage::Ping(b),
        Message::Pong(b) => AxumMessage::Pong(b),
        Message::Close(Some(close)) => AxumMessage::Close(Some(AxumFrame {
            code: close.code.into(),
            reason: close.reason.as_str().into(),
        })),
        Message::Close(None) => AxumMessage::Close(None),
        // Big problem if we reach this,
        // should't be something we encounter normally though
        Message::Frame(_) => unreachable!(),
    }
}

impl From<WebSocketStream<MaybeTlsStream<TcpStream>>> for WsConnection {
    fn from(value: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self::WebSocketClient(Box::new(value))
    }
}

impl From<WebSocket> for WsConnection {
    fn from(value: WebSocket) -> Self {
        Self::WebSocketServer(Box::new(value))
    }
}

#[cfg(feature = "mock_ws")]
impl From<MockWebSocket> for WsConnection {
    fn from(value: MockWebSocket) -> Self {
        Self::Mock(Box::new(value))
    }
}

impl WsConnection {
    /// # Errors
    ///
    /// This function errors if the underlying close implementation fails
    pub async fn close(&mut self) -> Result<(), anyhow::Error> {
        match self {
            WsConnection::WebSocketClient(ws) => ws.close().await?,
            WsConnection::WebSocketServer(ws) => ws.close().await?,
            #[cfg(feature = "mock_ws")]
            WsConnection::Mock(mock) => mock.close().await?,
        }

        Ok(())
    }
}

// Wrapper function for the traits, because of `Pin`
//
// lots of nasty stuff incoming, especially a bunch of `Into::into`,
// blame axum for that
impl WsConnection {
    // Stream

    fn poll_next_wr(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Message, anyhow::Error>>> {
        match self {
            WsConnection::WebSocketClient(ws) => ws.poll_next_unpin(cx).map_err(Into::into),
            WsConnection::WebSocketServer(ws) => ws
                .poll_next_unpin(cx)
                // grarly stuff
                .map(|v| v.map(|v| v.map(axum_to_tungstenite)))
                .map_err(Into::into),
            #[cfg(feature = "mock_ws")]
            WsConnection::Mock(mock) => mock.poll_next_unpin(cx).map_err(Into::into),
        }
    }

    // Sink

    fn poll_ready_wr(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), anyhow::Error>> {
        match self {
            WsConnection::WebSocketClient(ws) => ws.poll_ready_unpin(cx).map_err(Into::into),
            WsConnection::WebSocketServer(ws) => ws.poll_ready_unpin(cx).map_err(Into::into),
            #[cfg(feature = "mock_ws")]
            WsConnection::Mock(mock) => mock.poll_ready_unpin(cx).map_err(Into::into),
        }
    }

    fn start_send_wr(&mut self, item: Message) -> Result<(), anyhow::Error> {
        match self {
            WsConnection::WebSocketClient(ws) => ws.start_send_unpin(item).map_err(Into::into),
            WsConnection::WebSocketServer(ws) => ws
                .start_send_unpin(tungstenite_to_axum(item))
                .map_err(Into::into),
            #[cfg(feature = "mock_ws")]
            WsConnection::Mock(mock) => mock.start_send_unpin(item).map_err(Into::into),
        }
    }

    fn poll_flush_wr(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), anyhow::Error>> {
        match self {
            WsConnection::WebSocketClient(ws) => ws.poll_flush_unpin(cx).map_err(Into::into),
            WsConnection::WebSocketServer(ws) => ws.poll_flush_unpin(cx).map_err(Into::into),
            #[cfg(feature = "mock_ws")]
            WsConnection::Mock(mock) => mock.poll_flush_unpin(cx).map_err(Into::into),
        }
    }

    fn poll_close_wr(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), anyhow::Error>> {
        match self {
            WsConnection::WebSocketClient(ws) => ws.poll_close_unpin(cx).map_err(Into::into),
            WsConnection::WebSocketServer(ws) => ws.poll_close_unpin(cx).map_err(Into::into),
            #[cfg(feature = "mock_ws")]
            WsConnection::Mock(mock) => mock.poll_close_unpin(cx).map_err(Into::into),
        }
    }
}

impl Stream for WsConnection {
    type Item = Result<Message, anyhow::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_next_wr(cx)
    }
}

impl FusedStream for WsConnection {
    fn is_terminated(&self) -> bool {
        match self {
            WsConnection::WebSocketClient(ws) => ws.is_terminated(),
            WsConnection::WebSocketServer(ws) => ws.is_terminated(),
            #[cfg(feature = "mock_ws")]
            WsConnection::Mock(mock) => mock.is_terminated(),
        }
    }
}

impl Sink<Message> for WsConnection {
    type Error = anyhow::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_ready_wr(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        self.start_send_wr(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush_wr(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_close_wr(cx)
    }
}
