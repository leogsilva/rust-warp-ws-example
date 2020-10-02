
use std::sync::{Arc};
use warp::{Rejection, Filter, Reply};
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use std::convert::Infallible;
use warp::reply::json;
use warp::ws::Message;
use serde::{Deserialize, Serialize};
use warp::filters::sse::ServerSentEvent;
use tokio::time::interval;
use std::time::Duration;
use tokio::stream::StreamExt;

mod handler;
mod ws;

type Result<T> = std::result::Result<T, Rejection>;
type Clients = Arc<RwLock<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
pub struct Client {
    pub user_id: usize,
    pub topics: Vec<String>,
    sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>
}

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    user_id: usize,
}

#[derive(Serialize, Debug)]
pub struct RegisterResponse {
    url: String
}

#[derive(Deserialize, Debug)]
pub struct Event {
    topic: String,
    user_id: Option<usize>,
    message: String,
}

pub struct TopicsRequest {
    topics: Vec<String>,
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

async fn websocket_server() {
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));
    let health_route = warp::path("health").and_then(handler::health_handler);

    let register = warp::path("register");
    let register_routes = register
        .and(warp::post())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handler::register_handler)
        .or(register
            .and(warp::delete())
            .and(warp::path::param())
            .and(with_clients(clients.clone()))
            .and_then(handler::unregister_handler));

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with_clients(clients.clone()))
        .and_then(handler::ws_handler);

    let publish = warp::path!("publish")
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handler::publish_handler);

    let routes = health_route
        .or(register_routes)
        .or(ws_route)
        .or(publish)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn sse_counter(counter: u64) -> std::result::Result<impl ServerSentEvent, Infallible> {
    Ok(warp::sse::data(counter))
}

#[tokio::main]
async fn main() {
    let routes = warp::path("ticks")
        .and(warp::get())
        .map(|| {
            let mut counter: u64 = 0;
            let event_stream = interval(Duration::from_secs(1)).map(move |_| {
                counter += 1;
                sse_counter(counter)
            });
            warp::sse::reply(event_stream)
        })
        .with(warp::cors().allow_any_origin());
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

// #[tokio::main]
// async fn main() {
//     sse_server();
// }
