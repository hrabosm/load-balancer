use std::convert::Infallible;

use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::{BodyExt, Full};
use hyper::header::HeaderValue;
use hyper::{
    Request, Response, StatusCode, Uri,
    body::{Bytes, Incoming},
    header,
};
use hyper_util::client::legacy::{Client, connect::HttpConnector};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub async fn forward(
    req: Request<Incoming>,
    client: Client<HttpConnector, Incoming>,
    dest_url: &Uri,
) -> Result<Response<UnsyncBoxBody<Bytes, BoxError>>, Infallible> {
    let (mut parts, body) = req.into_parts();

    let mut uri_parts = dest_url.clone().into_parts();
    uri_parts.path_and_query = parts.uri.path_and_query().cloned();
    let new_uri: Uri;

    match Uri::from_parts(uri_parts) {
        Ok(uri) => new_uri = uri,
        Err(_) => {
            let body = Full::new(Bytes::from("Internal server error"))
                .map_err(|never| match never {})
                .boxed_unsync();

            let mut response = Response::new(body);
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

            return Ok(response);
        }
    }

    parts.uri = new_uri;

    let host = parts
        .uri
        .authority()
        .map(|authority| authority.as_str())
        .and_then(|authority| HeaderValue::from_str(authority).ok());

    if let Some(host) = host {
        parts.headers.insert(header::HOST, host);
    }

    let req = Request::from_parts(parts, body);
    match client.request(req).await {
        Ok(response) => {
            let (parts, body) = response.into_parts();
            let boxed_body = body
                .map_err(|err| -> BoxError { Box::new(err) })
                .boxed_unsync();
            Ok(Response::from_parts(parts, boxed_body))
        }
        Err(_) => {
            let body = Full::new(Bytes::from("Bad Gateway"))
                .map_err(|never| match never {})
                .boxed_unsync();

            let mut response = Response::new(body);
            *response.status_mut() = StatusCode::BAD_GATEWAY;

            Ok(response)
        }
    }
}