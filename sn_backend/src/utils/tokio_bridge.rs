use crossbeam::channel::Receiver as CrossbeamReceiver;
use futures::Stream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio_stream::wrappers::UnboundedReceiverStream;

pub struct TokenBridge<T> {
    stream: UnboundedReceiverStream<T>,
}

impl<T: Send + 'static> TokenBridge<T> {
    pub fn new(sync_rx: CrossbeamReceiver<T>) -> Self {
        let (tx, rx): (UnboundedSender<T>, UnboundedReceiver<T>) = unbounded_channel();

        // Bridge sync → async in a blocking task
        tokio::task::spawn_blocking(move || {
            for item in sync_rx {
                if tx.send(item).is_err() {
                    break;
                }
            }
            drop(tx); // Close the sender to signal end of stream
            println!("Bridge thread done — stream should end now.");
        });

        let stream = UnboundedReceiverStream::new(rx);
        Self { stream }
    }

    pub fn into_stream(self) -> impl Stream<Item = T> {
        self.stream
    }
}
