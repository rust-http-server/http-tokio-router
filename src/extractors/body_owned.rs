use std::ops::{Deref, DerefMut};

use super::FromRequest;
use crate::{error::HttpError, result::HttpResult};
use bytes::Bytes;
use futures::future::BoxFuture;
use serde::de::DeserializeOwned;
use http_tokio::{BodyReader, Request};

pub struct BodyOwned {
    bytes: Vec<u8>,
}

impl BodyOwned {
    pub fn json<T: DeserializeOwned>(self) -> HttpResult<T> {
        serde_json::from_str::<T>(&self.text()?).map_err(|e| HttpError::new(e.to_string(), 400))
    }

    pub fn text(self) -> HttpResult<String> {
        String::from_utf8(self.bytes).map_err(|e| HttpError::new(e.to_string(), 400))
    }

    pub fn bytes(self) -> Bytes {
        self.bytes.into()
    }
}

impl<'a> FromRequest<'a> for BodyOwned {
    type Future = BoxFuture<'a, HttpResult<Self>>;
    fn from_req(_: &'a Request, payload: &'a BodyReader) -> Self::Future {
        Box::pin(async move {
            let bytes = payload
                .read_all()
                .await
                .map_err(|err| HttpError::new(format!("io error reading body: {err}"), 500))?;
            if bytes.is_empty() {
                return Err(HttpError::new("found empty body".to_string(), 500));
            }
            Ok(BodyOwned { bytes })
        })
    }
}

#[derive(Debug)]
pub struct Json<T: DeserializeOwned, const ERR_CODE: u16 = 400>(T);

impl<T: DeserializeOwned, const ERR_CODE: u16> Json<T, ERR_CODE> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: DeserializeOwned, const ERR_CODE: u16> Deref for Json<T, ERR_CODE> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: DeserializeOwned, const ERR_CODE: u16> DerefMut for Json<T, ERR_CODE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: DeserializeOwned, const ERR_CODE: u16> FromRequest<'a> for Json<T, ERR_CODE> {
    type Future = BoxFuture<'a, HttpResult<Self>>;
    fn from_req(req: &'a Request, payload: &'a BodyReader) -> Self::Future {
        Box::pin(async move {
            let t = BodyOwned::from_req(req, payload)
                .await?
                .json::<T>()
                .map_err(|e| e.status(ERR_CODE))?;
            Ok(Json(t))
        })
    }
}
