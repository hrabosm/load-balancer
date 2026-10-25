use std::{convert::Infallible};

use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::{BodyExt, Full};
use hyper::{Request, Response, StatusCode, Uri, body::{Bytes, Incoming}};
use hyper_util::client::legacy::{Client, connect::HttpConnector};

const HYDRA_URL: &str = "http://192.168.10.51:4000";

type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub async fn forward(req: Request<Incoming>, client: Client<HttpConnector, Incoming>) -> Result<Response<UnsyncBoxBody<Bytes, BoxError>>, Infallible> {
    let(mut parts, body) = req.into_parts();

    let path_and_query: &str;

    match parts.uri.path_and_query() {
        Some(result) => path_and_query = result.as_str(),
        None => {
            let body = Full::new(Bytes::from("Internal server error"))
                .map_err(|never| match never {})
                .boxed_unsync();

            let mut response = Response::new(body);
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

            return Ok(response)
        }
    }

    let new_url = HYDRA_URL.to_owned() + path_and_query;
    let new_uri: Uri;

    match new_url.parse::<Uri>() {
        Ok(uri) => new_uri = uri,
        Err(_) => {
            let body = Full::new(Bytes::from("Internal server error"))
                .map_err(|never| match never {})
                .boxed_unsync();

            let mut response = Response::new(body);
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

            return Ok(response)
        }
    }

    parts.uri = new_uri;

    let req = Request::from_parts(parts, body);
    match client.request(req).await {
        Ok(response) => {
            let (parts, body) = response.into_parts();
            let boxed_body = body
                .map_err(|err| -> BoxError { Box::new(err) })
                .boxed_unsync();
            Ok(Response::from_parts(parts, boxed_body))
        },
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