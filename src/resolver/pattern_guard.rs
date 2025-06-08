use std::{cmp::Ordering, collections::HashMap};
use crate::{resolver::{ctx::ResolveContext, traits::Guard}, pattern::Pattern};

pub struct PatternGuard {
    pattern: Pattern,
}

impl PatternGuard {
    pub fn new(pattern: &str) -> Self {
        let pattern = Pattern::parse(pattern).unwrap();
        Self { pattern }
    }
}

impl Guard for PatternGuard {
    fn check<'a, 'ctx>(&self, ctx: &'a mut ResolveContext<'ctx>) -> bool {
        dbg!(&self.pattern);
        dbg!(&ctx.path_segments);

        if &ctx.req.method != &self.pattern.method && self.pattern.method != "ALL" {
            return false;
        }

        let mut params = HashMap::<String, String>::new();
        let mut segments = ctx.path_segments.clone();

        if self.pattern.chunks.len() == 0 { return true }
        
        for (i, guard_segment) in self.pattern.chunks.iter().enumerate() {
            let (head, tail) = match segments.split_first() {
                Some(pair) => pair,
                None => match i.cmp(&self.pattern.chunks.len()) {
                    Ordering::Less => return false,
                    _ => break
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

            if guard_segment == "*" {
                params.insert("*".to_string(), segments.join("/"));
                segments = Vec::with_capacity(0);
                break;
            }
            
            return false;
        }

        ctx.params.extend(params);
        ctx.path_segments = Vec::from(segments);

        true
    }
}