use std::{future::Future, pin::Pin};
use http_tokio::{RequestError, Response, StatusCode};

pub trait ServerEvents: Send + Sync {
    fn on_connection_error<'a>(&'a self, err: tokio::io::Error) {
        eprintln!("Connection error: {}", err);
    }
    fn handle_client_error(&self, err: RequestError, status_code: StatusCode) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        Box::pin(async move {
            Response::build().status(status_code).body(format!("invalid request: {err}"))
        })
    }
    fn handle_timeout(&self) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        Box::pin(async move {
            Response::build().status(StatusCode::REQUEST_TIMEOUT).header(("Connection", "close")).body("Request Timeout")
        })
    }
}

pub (crate) struct DefaultServerEvents;
impl ServerEvents for DefaultServerEvents {}