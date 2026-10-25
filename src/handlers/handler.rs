use std::convert::Infallible;

use hyper::{Request, Response, Uri, body::Incoming};
use hyper_util::client::legacy::{Client, connect::HttpConnector};

const HYDRA_URL: &str = "http://192.168.10.51:4000";

pub async fn forward(req: Request<hyper::body::Incoming>, client: Client<HttpConnector, Incoming>) -> Result<Response<Incoming>, Infallible> {
    let(mut parts, body) = req.into_parts();
    let path_and_query = parts.uri.path_and_query().unwrap().as_str();
    let new_url = HYDRA_URL.to_owned() + path_and_query;
    let new_uri = new_url.parse::<Uri>().unwrap();
    parts.uri = new_uri;

    let req = Request::from_parts(parts, body);
    let response = client.request(req).await.unwrap();
    Ok(response)
}