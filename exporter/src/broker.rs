use async_graphql::futures_util::Stream;
use once_cell::sync::Lazy;
use tokio::sync::broadcast::{channel, Sender};
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};

#[derive(Debug)]
pub struct EventBroker<E: Clone>(Lazy<Sender<E>>);

impl<E: std::fmt::Debug + Send + Sync + Clone + 'static> EventBroker<E> {
    pub const fn new() -> Self {
        Self(Lazy::new(|| channel::<E>(1024).0))
    }

    pub fn publish(&self, event: E) {
        let _ = self.0.send(event);
    }

    pub fn subscribe(&self) -> impl Stream<Item = Result<E, BroadcastStreamRecvError>> {
        BroadcastStream::new(self.0.subscribe())
    }
}
