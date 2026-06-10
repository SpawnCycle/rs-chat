use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Sink, SinkExt, Stream, StreamExt, stream::FusedStream};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{Error, Message, protocol::CloseFrame},
};

#[cfg(feature = "mock_ws")]
use crate::ws_mock::MockWebSocket;

#[derive(Debug)]
pub enum WsConnection {
    WebSocket(Box<WebSocketStream<MaybeTlsStream<TcpStream>>>),
    #[cfg(feature = "mock_ws")]
    Mock(Box<MockWebSocket>),
}

impl From<WebSocketStream<MaybeTlsStream<TcpStream>>> for WsConnection {
    fn from(value: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self::WebSocket(Box::new(value))
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
    pub async fn close(&mut self, frame: Option<CloseFrame>) -> Result<(), Error> {
        match self {
            WsConnection::WebSocket(ws) => WebSocketStream::close(ws, frame).await,
            #[cfg(feature = "mock_ws")]
            WsConnection::Mock(mock) => MockWebSocket::close(mock, frame).await,
        }
    }
}

// Wrapper function for the traits, because of `Pin`
impl WsConnection {
    // Stream

    fn poll_next_wr(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<Message, Error>>> {
        match self {
            WsConnection::WebSocket(ws) => ws.poll_next_unpin(cx),
            #[cfg(feature = "mock_ws")]
            WsConnection::Mock(mock) => mock.poll_next_unpin(cx),
        }
    }

    // Sink

    fn poll_ready_wr(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        match self {
            WsConnection::WebSocket(ws) => ws.poll_ready_unpin(cx),
            #[cfg(feature = "mock_ws")]
            WsConnection::Mock(mock) => mock.poll_ready_unpin(cx),
        }
    }

    fn start_send_wr(&mut self, item: Message) -> Result<(), Error> {
        match self {
            WsConnection::WebSocket(ws) => ws.start_send_unpin(item),
            #[cfg(feature = "mock_ws")]
            WsConnection::Mock(mock) => mock.start_send_unpin(item),
        }
    }

    fn poll_flush_wr(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        match self {
            WsConnection::WebSocket(ws) => ws.poll_flush_unpin(cx),
            #[cfg(feature = "mock_ws")]
            WsConnection::Mock(mock) => mock.poll_flush_unpin(cx),
        }
    }

    fn poll_close_wr(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        match self {
            WsConnection::WebSocket(ws) => ws.poll_close_unpin(cx),
            #[cfg(feature = "mock_ws")]
            WsConnection::Mock(mock) => mock.poll_close_unpin(cx),
        }
    }
}

impl Stream for WsConnection {
    type Item = Result<Message, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_next_wr(cx)
    }
}

impl FusedStream for WsConnection {
    fn is_terminated(&self) -> bool {
        match self {
            WsConnection::WebSocket(ws) => ws.is_terminated(),
            #[cfg(feature = "mock_ws")]
            WsConnection::Mock(mock) => mock.is_terminated(),
        }
    }
}

impl Sink<Message> for WsConnection {
    type Error = Error;

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
