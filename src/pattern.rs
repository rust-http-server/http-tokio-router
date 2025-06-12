use super::error::PatternError;
use std::fmt::{Debug, Display};

const ALLOWED_CHARS: [char; 5] = ['*', '{', '}', '_', '-'];

#[derive(Clone, Debug)]
pub struct Pattern {
    pub method: String,      // HTTP method (free-form, uppercase)
    pub full_path: String,   // The full path
    pub chunks: Vec<String>, // Path split into chunks
}

type PatternResult<T> = Result<T, PatternError>;

impl Pattern {
    pub fn new(method: impl AsRef<str>, path: impl AsRef<str>) -> Self {
        let method = method.as_ref().to_string();
        let full_path = path.as_ref().to_string();
        let chunks = full_path.split("/").map(|c| c.to_string()).collect();
        Self { method, full_path, chunks }
    }
    
    pub fn parse(input: &str) -> PatternResult<Self> {
        let mut method = "ALL".to_string();
        let path: &str;

        let (method_candidate, maybe_path) = if let Some((m, r)) = input.split_once(':') {
            (Some(m), r)
        } else {
            (None, input)
        };

        if let Some(m) = method_candidate {
            method = m.to_uppercase();
        }

        if maybe_path == "*" {
            return Ok(Self {
                method,
                full_path: "*".to_string(),
                chunks: vec!["*".to_string()],
            });
        }

        if !maybe_path.starts_with('/') {
            return Err(PatternError::PathMustStartWithSlash);
        }

        path = &maybe_path[1..]; // skip the initial slash
        let chunks = parse_path(path)?;
        let chunks = chunks.into_iter().map(String::from).collect();

        Ok(Self {
            method,
            full_path: format!("/{path}"),
            chunks,
        })
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.method, self.full_path)
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

fn parse_path(pattern: &str) -> PatternResult<Vec<&str>> {
    let pattern = pattern.trim_end_matches('/');

    if pattern.is_empty() {
        return Ok(Vec::with_capacity(0));
    }

    // let mut p_type = PatternType::Exact;
    let mut has_wildcard = false;
    let mut collected_chunks = Vec::new();

    for chunk in pattern.split('/') {
        if chunk.is_empty() {
            continue; // allow double slashes
        }

        if !is_valid_chunk(chunk) {
            return Err(PatternError::InvalidChars);
        }

        if has_wildcard {
            return Err(PatternError::WildcardPosition);
        }

        if chunk == "*" {
            has_wildcard = true;
            // p_type = PatternType::Dynamic;
        } else if chunk.starts_with('{') && chunk.ends_with('}') {
            let trimmed = &chunk[1..chunk.len() - 1];
            if trimmed.contains('{') || trimmed.contains('}') {
                return Err(PatternError::InvalidDynamic);
            }
        }

        collected_chunks.push(chunk);
    }

    Ok(collected_chunks)
}

fn is_valid_chunk(chunk: &str) -> bool {
    chunk
        .chars()
        .all(|c| c.is_alphanumeric() || ALLOWED_CHARS.contains(&c))
}