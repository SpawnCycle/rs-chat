use futures::SinkExt;
use futures::{Sink, Stream, stream::FusedStream};
use std::{collections::VecDeque, task::Poll};
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio_tungstenite::tungstenite::{Error, Message, protocol::CloseFrame};

use crate::consts::CHANNEL_BUFFER_SIZE;

#[derive(Debug)]
pub struct MockWebSocket {
    strategy: MockStrategy,
    tx: Sender<Message>,
    rx: Receiver<Message>,
    in_messages: VecDeque<Message>,
    out_messages: VecDeque<Message>,
    ended: bool,
    closing: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MockStrategy {
    Proxy,
    Store,
}

#[allow(unused)]
impl MockWebSocket {
    pub fn get_out(&self) -> &VecDeque<Message> {
        &self.out_messages
    }

    pub fn get_in(&self) -> &VecDeque<Message> {
        &self.in_messages
    }

    pub fn new_proxy(out_tx: Sender<Message>, out_rx: Receiver<Message>) -> Self {
        Self {
            strategy: MockStrategy::Proxy,
            tx: out_tx,
            rx: out_rx,
            in_messages: VecDeque::new(),
            out_messages: VecDeque::new(),
            ended: false,
            closing: false,
        }
    }

    pub fn new_store() -> Self {
        let (tx, rx) = channel(CHANNEL_BUFFER_SIZE);
        Self {
            strategy: MockStrategy::Store,
            tx,
            rx,
            in_messages: VecDeque::new(),
            out_messages: VecDeque::new(),
            ended: false,
            closing: false,
        }
    }

    pub async fn close(&mut self, frame: Option<CloseFrame>) -> Result<(), Error> {
        self.out_messages.push_back(Message::Close(frame));
        if self.strategy == MockStrategy::Proxy {
            let _ = self.flush().await;
        }

        Ok(())
    }

    fn flush_out_messages(&mut self) {
        if self.strategy == MockStrategy::Proxy {
            for msg in &self.out_messages {
                let _ = self.tx.blocking_send(msg.clone());
            }
            self.out_messages.clear();
        }
    }

    fn poll_in_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            self.in_messages.push_back(msg);
        }
    }
}

impl Stream for MockWebSocket {
    type Item = Result<Message, Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if self.ended {
            Poll::Ready(Some(Err(Error::AlreadyClosed)))
        } else {
            self.poll_in_messages();
            Poll::Ready(self.in_messages.pop_front().map(Ok))
        }
    }
}

impl FusedStream for MockWebSocket {
    fn is_terminated(&self) -> bool {
        self.ended
    }
}

impl Sink<Message> for MockWebSocket {
    type Error = Error;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: std::pin::Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        self.out_messages.push_back(item);

        Ok(())
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.flush_out_messages();

        Poll::Ready(Ok(()))
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        if self.closing {
            self.flush_out_messages();
        } else {
            self.out_messages.push_back(Message::Close(None));
        }

        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn print_messages(mock: &MockWebSocket) {
        eprintln!("IN = {:?}", mock.get_in());
        eprintln!("OUT = {:?}", mock.get_out());
    }

    #[tokio::test]
    async fn mock_ws_store_test() -> anyhow::Result<()> {
        let mut mock = MockWebSocket::new_store();

        let msg = Message::Text("Hello".into());
        mock.send(msg.clone()).await?;

        print_messages(&mock);
        assert!(mock.get_out().contains(&msg));

        Ok(())
    }
}
