use crate::middleware::MiddlewareStack;
use http_tokio::Request;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ResolveContext<'a> {
    pub req: &'a Request,
    pub(crate) path_segments: Vec<String>,
    pub(crate) params: HashMap<String, String>,
    pub(crate) layers: MiddlewareStack,
}

impl<'a> ResolveContext<'a> {
    pub fn new(req: &'a Request) -> Self {
        let path_segments = req.path.split("/").filter(|c| !c.is_empty()).map(|c| c.to_string()).collect();
        ResolveContext {
            req,
            path_segments,
            params: HashMap::new(),
            layers: MiddlewareStack::new(),
        }
    }

    pub fn add_param(&mut self, key: String, value: String) {
        self.params.insert(key, value);
    }

    pub fn nest(&mut self, path_segments: Vec<String>, params: HashMap<String, String>, more_layers: MiddlewareStack) -> ResolveContext<'a> {
        let mut layers = self.layers.clone();
        layers.extend(more_layers);
        ResolveContext { req: self.req, path_segments, params, layers }
    }

    pub fn absorb(&mut self, another: ResolveContext<'a>) {
        self.path_segments = another.path_segments.clone();
        self.params = another.params.clone();
        self.layers = another.layers.clone();
    }
}
