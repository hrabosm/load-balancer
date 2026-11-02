mod handlers;
mod backend;
mod strategies;

use std::net::SocketAddr;
use std::sync::Arc;

use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpListener;

use crate::backend::node::Node;
use crate::backend::pool::Pool;
use crate::handlers::handler::forward;
use crate::strategies::round_robin::RoundRobin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = TcpListener::bind(addr).await?;

    let client: Client<HttpConnector, Incoming> =
        Client::builder(TokioExecutor::new()).build_http();

    let backend_nodes = vec!(
        Node::from_str("http://192.168.10.51:4000").unwrap(),
        Node::from_str("http://192.168.10.51:4001").unwrap(),
        Node::from_str("http://192.168.10.51:4002").unwrap(),
        Node::from_str("http://192.168.10.51:4003").unwrap(),
    );

    let backend_pool = Arc::new(
        Pool::new(backend_nodes, RoundRobin::new())
            .map_err(std::io::Error::other)?
    );

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        let client = client.clone();
        let pool = Arc::clone(&backend_pool);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(|req| forward(req, client.clone(), pool.select_backend().uri())),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}