use std::{future::Future, pin::Pin};

use http_tokio::{RequestError, Response, StatusCode};

use crate::server::events::ServerEvents;

pub struct ServerEventsBuilder {
    on_connection_error: Option<Box<dyn Fn(tokio::io::Error) + Send + Sync>>,
    handle_client_error: Option<Box<dyn Fn(RequestError, StatusCode) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>>,
    handle_timeout: Option<Box<dyn Fn() -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>>,
}

impl ServerEventsBuilder {
    pub fn new() -> Self {
        ServerEventsBuilder {
            on_connection_error: None,
            handle_client_error: None,
            handle_timeout: None,
        }
    }

    pub fn on_connection_error<F>(mut self, f: F) -> Self
    where
        F: Fn(tokio::io::Error) + Send + Sync + 'static,
    {
        self.on_connection_error = Some(Box::new(f));
        self
    }

    pub fn handle_client_error<F>(mut self, f: F) -> Self
    where
        F: Fn(RequestError, StatusCode) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync + 'static,
    {
        self.handle_client_error = Some(Box::new(f));
        self
    }

    pub fn handle_timeout<F>(mut self, f: F) -> Self
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync + 'static,
    {
        self.handle_timeout = Some(Box::new(f));
        self
    }
}

impl ServerEvents for ServerEventsBuilder {
    fn on_connection_error<'a>(&'a self, err: tokio::io::Error) {
        if let Some(ref f) = self.on_connection_error {
            f(err);
        } else {
            ServerEvents::on_connection_error(self, err);
        }
    }

    fn handle_client_error(&self, err: RequestError, status_code: StatusCode) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        if let Some(ref f) = self.handle_client_error {
            f(err, status_code)
        } else {
            ServerEvents::handle_client_error(self, err, status_code)
        }
    }

    fn handle_timeout(&self) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        if let Some(ref f) = self.handle_timeout {
            f()
        } else {
            ServerEvents::handle_timeout(self)
        }
    }
}
