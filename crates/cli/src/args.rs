//! Typed access to a request's JSON `args`. Every failure is an
//! `invalid_request` (or `invalid_id`) that points at `describe <verb>`.

use std::str::FromStr;

use serde_json::{Map, Value};
use shelfmark_core::id::IdError;
use shelfmark_core::{Error, ErrorCode, Verb};

/// A bad request for `verb`, with the describe command as the fix.
pub fn bad(verb: Verb, message: impl Into<String>) -> Error {
    Error::new(ErrorCode::InvalidRequest, message)
        .next(format!("shelfmark describe {}", verb.as_str()))
}

pub struct Args<'a> {
    verb: Verb,
    map: &'a Map<String, Value>,
}

impl<'a> Args<'a> {
    /// `args` must be an object whose keys are all in `known`.
    pub fn new(verb: Verb, args: &'a Value, known: &[&str]) -> Result<Self, Error> {
        let Value::Object(map) = args else {
            return Err(bad(verb, "args must be a JSON object"));
        };
        if let Some(k) = map.keys().find(|k| !known.contains(&k.as_str())) {
            return Err(bad(
                verb,
                format!(
                    "`{}` takes no argument `{k}` (it takes: {})",
                    verb.as_str(),
                    if known.is_empty() {
                        "nothing".to_string()
                    } else {
                        known.join(", ")
                    }
                ),
            ));
        }
        Ok(Args { verb, map })
    }

    pub fn verb(&self) -> Verb {
        self.verb
    }

    fn get(&self, key: &str) -> Option<&'a Value> {
        self.map.get(key).filter(|v| !v.is_null())
    }

    pub fn str(&self, key: &str) -> Result<Option<&'a str>, Error> {
        match self.get(key) {
            None => Ok(None),
            Some(Value::String(s)) => Ok(Some(s)),
            Some(_) => Err(bad(self.verb, format!("`{key}` must be a string"))),
        }
    }

    pub fn require(&self, key: &str) -> Result<&'a str, Error> {
        self.str(key)?
            .ok_or_else(|| bad(self.verb, format!("`{key}` is required")))
    }

    pub fn f64(&self, key: &str) -> Result<Option<f64>, Error> {
        match self.get(key) {
            None => Ok(None),
            Some(v) => v
                .as_f64()
                .map(Some)
                .ok_or_else(|| bad(self.verb, format!("`{key}` must be a number"))),
        }
    }

    pub fn usize(&self, key: &str) -> Result<Option<usize>, Error> {
        match self.get(key) {
            None => Ok(None),
            Some(v) => v
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .map(Some)
                .ok_or_else(|| bad(self.verb, format!("`{key}` must be a non-negative integer"))),
        }
    }

    /// A string or an array of strings.
    pub fn strings(&self, key: &str) -> Result<Vec<String>, Error> {
        let err = || {
            bad(
                self.verb,
                format!("`{key}` must be a string or a list of strings"),
            )
        };
        match self.get(key) {
            None => Ok(Vec::new()),
            Some(Value::String(s)) => Ok(vec![s.clone()]),
            Some(Value::Array(a)) => a
                .iter()
                .map(|v| v.as_str().map(str::to_string).ok_or_else(err))
                .collect(),
            Some(_) => Err(err()),
        }
    }

    /// A typed ID argument; a malformed one is `invalid_id`.
    pub fn id<T: FromStr<Err = IdError>>(&self, key: &str) -> Result<Option<T>, Error> {
        self.str(key)?
            .map(|s| s.parse::<T>().map_err(|e| Error::invalid_id(&e)))
            .transpose()
    }

    pub fn require_id<T: FromStr<Err = IdError>>(&self, key: &str) -> Result<T, Error> {
        self.id(key)?
            .ok_or_else(|| bad(self.verb, format!("`{key}` is required")))
    }
}
