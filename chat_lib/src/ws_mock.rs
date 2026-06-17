use std::{collections::VecDeque, task::Poll};

use futures::{Sink, SinkExt, Stream, stream::FusedStream};
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio_tungstenite::tungstenite::{Error, Message, protocol::CloseFrame};

#[derive(Debug)]
pub struct MockWebSocket {
    strategy: MockStrategy,
    tx: Sender<Message>,
    rx: Receiver<Message>,
    /// Stores the incoming messages,
    /// which can be added through the sender
    in_messages: VecDeque<Message>,
    /// Stores the outgoing messages (through the Sink trait)
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
    #[must_use]
    pub fn get_out(&self) -> &VecDeque<Message> {
        &self.out_messages
    }

    #[must_use]
    pub fn get_in(&self) -> &VecDeque<Message> {
        &self.in_messages
    }

    #[must_use]
    pub fn new_proxy(out_tx: Sender<Message>, in_rx: Receiver<Message>) -> Self {
        Self {
            strategy: MockStrategy::Proxy,
            tx: out_tx,
            rx: in_rx,
            in_messages: VecDeque::new(),
            out_messages: VecDeque::new(),
            ended: false,
            closing: false,
        }
    }

    #[must_use]
    pub fn new_store() -> Self {
        // the size doesn't actually matter here because it's unused either way
        let (tx, rx) = channel(32);
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

    /// # Errors
    ///
    /// This function errors if there was an error sending the closing frame
    pub async fn close(&mut self, frame: Option<CloseFrame>) -> Result<(), Error> {
        self.out_messages.push_back(Message::Close(frame));
        if self.strategy == MockStrategy::Proxy {
            let _ = self.flush().await;
        }

        Ok(())
    }

    fn flush_out_messages(&mut self) {
        if self.strategy == MockStrategy::Proxy {
            let mut send_failed = false;
            // send the messages and remove those which succeeded
            self.out_messages.retain(|msg| {
                let res = self.tx.try_send(msg.clone());
                res.is_err()
            });
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
    #![allow(
        clippy::print_stdout,
        clippy::print_stderr,
        reason = "This doesn't apply to tests"
    )]

    use super::*;

    use futures::StreamExt;
    use tokio::sync::mpsc::channel;

    #[allow(unused)]
    fn print_messages(mock: &MockWebSocket) {
        eprintln!("IN = {:?}", mock.get_in());
        eprintln!("OUT = {:?}", mock.get_out());
    }

    #[tokio::test]
    async fn mock_ws_store_single_message() -> anyhow::Result<()> {
        let mut mock = MockWebSocket::new_store();

        let msg = Message::text("Hello");
        mock.send(msg.clone()).await?;

        print_messages(&mock);
        assert!(mock.get_out().contains(&msg));

        Ok(())
    }

    #[tokio::test]
    async fn mock_ws_proxy_single_message() -> anyhow::Result<()> {
        let (in_tx, mut in_rx) = channel(32);
        let (out_tx, out_rx) = channel(32);
        let mut mock = MockWebSocket::new_proxy(in_tx, out_rx);

        // Incoming message, which would be accessed through `Stream`
        let in_msg = Message::text("IN");
        // Outgoing message, which would be sent through `Sink`
        let out_msg = Message::text("OUT");

        // Send an incoming message
        out_tx.send(in_msg.clone()).await?;
        // Send an outgoing message
        mock.send(out_msg.clone()).await?;

        // gather the incoming messages
        let out = mock
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        assert!(out.contains(&in_msg));
        assert!(in_rx.recv().await == Some(out_msg));

        Ok(())
    }
}
