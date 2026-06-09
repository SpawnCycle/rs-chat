use futures::{Sink, Stream, stream::FusedStream};
use std::{collections::VecDeque, task::Poll};
use tokio_tungstenite::tungstenite::{self, Error, Message};

// TODO: finish implementing

#[derive(Debug, Clone)]
pub struct MockWebSocket {
    in_messages: VecDeque<tungstenite::Message>,
    out_messages: VecDeque<tungstenite::Message>,
    ended: bool,
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
            Poll::Ready(self.out_messages.pop_front().map(|i| Ok(i)))
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
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: std::pin::Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        self.out_messages.push_back(item);

        Ok(())
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.out_messages.push_back(Message::Close(None));

        Poll::Ready(Ok(()))
    }
}
