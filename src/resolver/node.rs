use std::{cmp::Ordering, collections::HashMap, sync::Arc};
use crate::{middleware::{Middleware, MiddlewareStack}, pattern::Pattern, resolver::{ctx::ResolveContext, traits::{Handler, Resolver}}};

pub struct Node {
    // guards: Vec<Box<dyn Guard>>,
    pattern: Pattern,
    layers: MiddlewareStack,
    childs: Vec<Box<dyn Resolver>>,
}

impl Resolver for Node {
    fn resolve<'a, 'ctx>(&'ctx self, ctx: &'a mut ResolveContext<'ctx>) -> Option<&'ctx dyn Handler> {
        if &ctx.req.method != &self.pattern.method && self.pattern.method != "ALL" {
            return None;
        }

        match self.check_path(ctx) {
            Some((updated_params, updated_segments)) => {
                let mut nested_ctx = ctx.nest(updated_segments, updated_params, self.layers.clone());
                match self.childs.iter().find_map(|node| node.resolve(&mut nested_ctx)) {
                    Some(child) => {
                        ctx.absorb(nested_ctx);
                        Some(child)
                    },
                    None => None,
                }
            },
            None => None,
        }
    }
}

impl Node {
    fn check_path<'a, 'ctx>(&'ctx self, ctx: &'a mut ResolveContext<'ctx>) -> Option<(HashMap<String, String>, Vec<String>)> {
        let mut segments = ctx.path_segments.clone();
        let mut params = HashMap::<String, String>::new();

        if self.pattern.chunks.len() == 0 { return Some((params, segments)) }


        for (i, guard_segment) in self.pattern.chunks.iter().enumerate() {
            if guard_segment == "*" {
                params.insert("*".to_string(), segments.join("/"));
                segments = Vec::with_capacity(0);
                break;
            }

            let (head, tail) = match segments.split_first() {
                Some(pair) => pair,
                None => match i.cmp(&self.pattern.chunks.len()) {
                    Ordering::Greater => break,
                    _ => return None,
                }
            };

            if guard_segment.starts_with("{") && guard_segment.ends_with("}") {
                let param = guard_segment[1..guard_segment.len() - 1].to_string();
                params.insert(param, head.to_string());
                segments = tail.to_vec();
                continue;
            }

            if guard_segment == head {
                segments = tail.to_vec();  
                continue;
            }

            return None;
        }

        Some((params, segments))
    }
}



impl Node {
    pub (crate) fn new() -> Node {
        Node {
            childs: Vec::new(),
            layers: Vec::new(),
            pattern: Pattern::parse("ALL:/").unwrap()
        }
    }

    pub (crate) fn with_pattern(pattern: &str) -> Node {
        Node {
            childs: Vec::new(),
            layers: Vec::new(),
            pattern: Pattern::parse(pattern).unwrap()
        }
    }

    pub fn add(mut self, srv: impl Resolver) -> Self {
        self.childs.push(Box::new(srv));
        self
    }

    pub fn at(self, pattern: &str, srv: impl Resolver) -> Self {
        self.add(helpers::scope(pattern).add(srv))
    }

    pub fn wrap(mut self, middleware: impl Middleware) -> Self {
        self.layers.push(Arc::new(middleware));
        self
    }
}


pub mod helpers {
    use crate::resolver::traits::Handler;
    use super::Node;

    pub fn scope(pattern: &str) -> Node {
        Node::with_pattern(pattern)
    }

    macro_rules! methods_helpers {
        ($(($fn:ident, $method:expr))+) => {
            $(
                pub fn $fn(handler: impl Handler) -> Node {
                    scope(&(format!("{}:/", $method))).add(handler)
                }
            )+
        };
    }

    methods_helpers!{
        (get, "GET")
        (post, "POST")
        (put, "PUT")
        (patch, "PATCH")
        (delete, "DELETE")
    }
}