use crate::error::{ErrorBackend, ResultAPIStream};
use crate::utils::tokio_bridge::TokenBridge;
use axum::body::Body;
use axum::response::{IntoResponse, Response};
use crossbeam::channel::Receiver;
use futures::StreamExt;
use sn_core::types::stream_data::StreamData;

/// A builder for constructing a Server-Sent Events (SSE) HTTP response
/// from a stream of `StreamData` received over a bounded channel.
///
/// This builder abstracts the logic of transforming internal data into an
/// SSE-compatible response, encoding each `StreamData` as a `data: ...\n\n`
/// formatted string and streaming it using an `axum::body::Body`.
///
/// Typical usage:
/// ```rust
/// let (tx, rx) = bounded::<StreamData>(100);
/// let response = SseResponseBuilder::new(rx).build();
/// ```
pub struct SseResponseBuilder {
    rx: Receiver<StreamData>,
}

impl SseResponseBuilder {
    /// Creates a new `SseResponseBuilder` with the given `Receiver<StreamData>`.
    ///
    /// # Arguments
    /// * `rx` - A bounded channel receiver from which `StreamData` will be streamed.
    pub fn new(rx: Receiver<StreamData>) -> Self {
        Self { rx }
    }

    /// Builds the final SSE-compatible HTTP response.
    ///
    /// Converts the `StreamData` stream into Server-Sent Event format and wraps it
    /// in a streaming HTTP `Response` with the appropriate `Content-Type` header.
    ///
    /// # Returns
    /// * `ResultAPIStream` — a streaming HTTP response ready to be returned from an axum handler.
    ///
    /// # ErrorBackends
    /// Returns a `FailedBuildSSEResponse` error if the HTTP response construction fails.
    pub fn build(self) -> ResultAPIStream {
        let bridge = TokenBridge::new(self.rx);
        let stream = bridge
            .into_stream()
            .map(|data| Ok::<_, ErrorBackend>(format!("data: {}\n\n", data.to_json())));

        let body = Body::from_stream(stream);

        Ok(Response::builder()
            .header("Content-Type", "text/event-stream")
            .body(body)
            .map_err(|e| ErrorBackend::FailedBuildSSEResponse(e.to_string()))
            .into_response())
    }
}
