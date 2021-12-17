use ::std::collections::HashMap;
use ::std::collections::HashSet;
use ::std::fmt;
use ::std::fs::read_to_string;
use ::std::future::Future;
use ::std::hash;
use ::std::path::Path;
use ::std::path::PathBuf;

use ::derive_getters::Getters;
use ::derive_new::new;
use ::futures::{FutureExt, stream, StreamExt, TryFutureExt, TryStreamExt};
use ::lazy_static::lazy_static;
use ::log::{debug, info, warn};
use ::regex::Regex;
use ::reqwest::Client;

#[derive(Debug, Getters, new)]
pub struct Dockerfile {
    path: PathBuf,
    content: String,
}

#[derive(Debug, Eq, Getters, new)]
pub struct Parent {
    name: String,
    version: String,
    suffix: String,
}

impl From<(&str, &str, &str)> for Parent {
    fn from(parts: (&str, &str, &str)) -> Self {
        Parent {
            name: parts.0.to_string(),
            version: parts.1.to_string(),
            suffix: parts.2.to_string(),
        }
    }
}

impl fmt::Display for Parent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.suffix.is_empty() {
            write!(f, "{}:{}", &self.name, &self.version)
        } else {
            write!(f, "{}:{} {}", &self.name, &self.version, &self.suffix)
        }
    }
}

impl PartialEq for Parent {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version == other.version
    }
}

impl hash::Hash for Parent {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes());
        state.write(self.version.as_bytes());
    }
}
