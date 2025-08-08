use bytes::Bytes;
use futures_util::{Stream, StreamExt};

/// Converts a stream of `Bytes` into a stream of string lines using an `mpsc` channel.
///
/// This function is typically used to handle streaming responses (e.g., from an HTTP request),
/// where each chunk of data might not represent a full line. It accumulates partial chunks
/// into a buffer until it finds a newline (`\n`), then sends each full line through an async channel.
///
/// # Arguments
/// * `stream` - A `Stream` of `Result<Bytes, reqwest::Error>`, typically from an HTTP response.
///
/// # Returns
/// * `Receiver<String>` - An async receiver that yields complete string lines from the stream.
///
/// # Behavior
/// - Spawns a background task to process the incoming byte chunks.
/// - Gracefully skips malformed UTF-8 or errored chunks.
/// - Uses newline (`\n`) to separate lines, supports partial buffering.
pub async fn stream_response_bytes(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> tokio::sync::mpsc::Receiver<String> {
    let mut stream = Box::pin(stream);
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(32);
    tokio::spawn(async move {
        let mut buffer = String::new();
        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error receiving chunk: {}", e);
                    continue; // Skip this chunk on error
                }
            };

            let chunk_str = match String::from_utf8(chunk.to_vec()) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error converting chunk to string: {}", e);
                    continue; // Skip this chunk on error
                }
            };
            buffer.push_str(chunk_str.as_str());

            while let Some(pos) = buffer.rfind('\n') {
                let line = buffer[..pos].trim();
                let _ = tx.send(line.to_string()).await;
                // prepare next buffer
                buffer = buffer[pos + 1..].to_string();
            }
        }
    });
    rx
}

#[cfg(test)]
use futures_util::stream;
#[tokio::test]

async fn test_stream_response_bytes() {
    let data_chunks = vec![
        Ok(Bytes::from("{\"key\": \"value\"}\n")),
        // Simulating chunks not being transmitted in one go
        Ok(Bytes::from("{\"another_key\":")),
        Ok(Bytes::from("\"another_value\"}\n")),
    ];

    // Create a mock stream from the test data
    let mock_stream = stream::iter(data_chunks);

    // Call the function
    let mut rx = stream_response_bytes(mock_stream).await;

    // Collect output lines
    let mut output = vec![];
    while let Some(line) = rx.recv().await {
        output.push(line);
    }

    assert_eq!(
        output,
        vec![
            "{\"key\": \"value\"}",
            "{\"another_key\":\"another_value\"}",
        ]
    );
}
