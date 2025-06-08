use super::error::HttpError;
use futures::future::BoxFuture;
use http_tokio::{content_type::ContentType, Response};

pub type HttpResult<T> = Result<T, HttpError>;
pub type RouteResult = HttpResult<Response>;
pub type HandlerResult<'a> = BoxFuture<'a, RouteResult>;

pub trait IntoRouteResult {
    fn into(self) -> RouteResult;
}

impl IntoRouteResult for Response {
    fn into(self) -> RouteResult {
        Ok(self)
    }
}

impl IntoRouteResult for &'static str {
    fn into(self) -> RouteResult {
        Ok(Response::build().body(self))
    }
}

impl IntoRouteResult for serde_json::Value {
    fn into(self) -> RouteResult {
        Ok(Response::build().content_type(ContentType::Json).body(self.to_string()))
    }
}

impl<T: IntoRouteResult, E: std::error::Error> IntoRouteResult for Result<T, E> {
    fn into(self) -> RouteResult {
        match self {
            Ok(val) => val.into(),
            Err(e) => Err(HttpError::err(e))
        }
    }
}

impl<T: IntoRouteResult> IntoRouteResult for Result<T, HttpError> {
    fn into(self) -> RouteResult {
        match self {
            Ok(val) => val.into(),
            Err(e) => Err(e)
        }
    }
}