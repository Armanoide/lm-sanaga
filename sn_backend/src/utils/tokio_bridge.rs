use crossbeam::channel::Receiver as CrossbeamReceiver;
use futures::Stream;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio_stream::wrappers::UnboundedReceiverStream;

/// A utility struct that bridges a synchronous crossbeam channel (`Receiver<T>`)
/// into an asynchronous stream (`impl Stream<Item = T>`), enabling integration
/// of sync data sources into async contexts.
///
/// This is particularly useful when you have a sync thread (like running in a
/// blocking context or external thread) and need to consume its output using
/// asynchronous streaming interfaces (like with `axum`, or `futures`).
///
/// Internally, this spawns a blocking task using `tokio::task::spawn_blocking`
/// which reads from the sync channel and forwards items to an async `mpsc` channel.
///
/// # Type Parameters
/// - `T`: The item type sent over the channel. Must be `Send + 'static`.
///
/// # Example
/// ```rust
/// use crossbeam::channel::bounded;
///
/// let (tx, rx) = bounded::<String>(100);
/// let bridge = TokenBridge::new(rx);
/// let mut stream = bridge.into_stream();
///
/// // `stream` is now an async stream of `String` values
/// ```
pub struct TokenBridge<T> {
    stream: UnboundedReceiverStream<T>,
}

impl<T: Send + 'static> TokenBridge<T> {
    /// Creates a new `TokenBridge` from a synchronous `crossbeam_channel::Receiver<T>`.
    ///
    /// Spawns a blocking task that reads from the provided sync receiver and
    /// forwards all received items into an async unbounded channel.
    ///
    /// # Arguments
    /// * `sync_rx` - The synchronous crossbeam receiver from which data will be consumed.
    ///
    /// # Returns
    /// * A `TokenBridge` instance containing an async-compatible stream.
    pub fn new(sync_rx: CrossbeamReceiver<T>) -> Self {
        let (tx, rx): (UnboundedSender<T>, UnboundedReceiver<T>) = unbounded_channel();

        // Bridge sync → async in a blocking task
        tokio::task::spawn_blocking(move || {
            for item in sync_rx {
                if tx.send(item).is_err() {
                    break;
                }
            }
        });

        let stream = UnboundedReceiverStream::new(rx);
        Self { stream }
    }

    /// Converts the `TokenBridge` into an async stream.
    ///
    /// This stream yields each item originally received on the sync channel.
    ///
    /// # Returns
    /// * `impl Stream<Item = T>` - An asynchronous stream of items.
    pub fn into_stream(self) -> impl Stream<Item = T> {
        self.stream
    }
}
