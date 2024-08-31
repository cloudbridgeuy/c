use eventsource_client as es;
use futures::stream::Stream;

use crate::error::Error;

pub type Json = serde_json::Value;
pub type ApiResult<T> = Result<T, crate::error::Error>;

pub trait Requests {
    /// # Errors
    ///
    /// Will return `Err` if the POST request fails, or we are unable to deserialize the response.
    fn post(&self, sub_url: &str, body: Json) -> ApiResult<Json>;
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

pub(crate) fn deal_response(
    response: Result<ureq::Response, ureq::Error>,
    sub_url: &str,
) -> ApiResult<Json> {
    match response {
        Ok(resp) => {
            let json = match resp.into_json::<Json>() {
                Ok(json) => json,
                Err(e) => {
                    log::error!("deserializing error from: {sub_url}, error: {e}");
                    return Err(Error::IO(e));
                }
            };
            log::debug!("deserialized response from {sub_url}: {json}");
            Ok(json)
        }
        Err(err) => match err {
            ureq::Error::Status(status, response) => {
                let error_msg = match response.into_json::<Json>() {
                    Ok(json) => json.to_string(),
                    Err(e) => format!("Unable to deserialize error: {e}"),
                };
                log::error!(
                    "error response from {sub_url} with status: {status}, error: {error_msg}"
                );
                Err(Error::ApiError(format!("{error_msg}")))
            }
            ureq::Error::Transport(e) => {
                log::error!(
                    "transport response error from {sub_url}: {:?}",
                    e.to_string()
                );
                Err(Error::ApiError(e.to_string()))
            }
        },
    }
}

pub(crate) fn tail(client: &impl es::Client) -> impl Stream<Item = Result<es::SSE, es::Error>> {
    client.stream()
}
