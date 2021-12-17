use ::std::collections::HashMap;
use ::std::collections::HashSet;
use ::std::fmt;
use ::std::fs::read_to_string;
use ::std::future::Future;
use ::std::hash;
use ::std::path::Path;
use ::std::path::PathBuf;

use ::futures::{FutureExt, stream, StreamExt, TryFutureExt, TryStreamExt};
use ::lazy_static::lazy_static;
use ::log::{debug, info, warn};
use ::regex::Regex;
use ::reqwest::Client;

lazy_static! {
    static ref FROM_RE: Regex = Regex::new(r"^FROM\s+(\S+):(\S+)\s*(.*)$").unwrap();
}

pub async fn read_all_dockerfiles(dockerfiles: &[PathBuf]) -> Result<Vec<Dockerfile>, String> {
    //TODO @mark: async
    dockerfiles.iter()
        .map(|path| read_dockerfile(path.as_path()))
        .collect::<Result<Vec<_>, _>>()
}

fn read_dockerfile(path: &Path) -> Result<Dockerfile, String> {
    info!("reading dockerfile: {}", path.to_string_lossy());
    match read_to_string(path) {
        Ok(content) => Ok(Dockerfile {
            path: path.to_path_buf(),
            content,
        }),
        Err(err) => Err(format!("Could not read Dockerfile '{}'.\n\
            Provide a correct path using -f PATH.\nError: {}", path.to_string_lossy(), err)),
    }
}

pub fn extract_parents(dockerfiles: &[Dockerfile]) -> HashSet<Parent> {
    dockerfiles.iter()
        .flat_map(|file| file.content.lines())
        .filter(|line| line.starts_with("FROM "))
        .flat_map(|line| parse_line_from(line).into_iter())
        .inspect(|parent| debug!("found parent: {}", &parent))
        .collect::<HashSet<_>>()
}

fn parse_line_from(line: &str) -> Option<Parent> {
    match FROM_RE.captures(line) {
        Some(matches) => Some(Parent::from((&matches[1], &matches[2], &matches[3]))),
        None => {
            if line.contains(":") {
                warn!("warning: FROM line, but could not recognize version:\n  {}", line);
            } else {
                info!("skipping line because there is no version: {}", line);
            }
            None
        }
    }
}
