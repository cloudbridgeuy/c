use eventsource_client as es;
use futures::stream::Stream;

pub type Json = serde_json::Value;

pub trait Requests {
    /// # Errors
    ///
    /// Will return `Err` if:
    ///
    /// - The headers can't be loaded to the request.
    /// - The body can't be loaded to the request.
    /// - The POST request to start the stream fails.
    /// - The stream connection fails to reconnect.
    /// - A stream can't be created.
    fn post_stream(
        &self,
        sub_url: &str,
        body: Json,
    ) -> Result<impl Stream<Item = Result<es::SSE, es::Error>>, es::Error>;
}

pub(crate) fn tail(client: &impl es::Client) -> impl Stream<Item = Result<es::SSE, es::Error>> {
    client.stream()
}
