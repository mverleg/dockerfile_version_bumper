use ::std::collections::HashSet;
use ::std::fs::read_to_string;
use ::std::path::Path;
use ::std::path::PathBuf;

use ::futures::{FutureExt, stream, StreamExt, TryFutureExt, TryStreamExt};
use ::lazy_static::lazy_static;
use ::log::{debug, info, warn};
use ::regex::Regex;

use crate::Parent;

use super::data::Dockerfile;

lazy_static! {
    static ref FROM_RE: Regex = Regex::new(r"^FROM\s+(\S+):(\S+)\s*(.*)$").unwrap();
    static ref TAG_DIGITS_RE: Regex = Regex::new(r"^[0-9]+$").unwrap();
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
        Ok(content) => Ok(Dockerfile::new(path.to_path_buf(), content)),
        Err(err) => Err(format!("Could not read Dockerfile '{}'.\n\
            Provide a correct path using -f PATH.\nError: {}", path.to_string_lossy(), err)),
    }
}

pub fn extract_parents(dockerfiles: &[Dockerfile]) -> HashSet<Parent> {
    dockerfiles.iter()
        .flat_map(|file| file.content().lines())
        .filter(|line| line.starts_with("FROM "))
        .flat_map(|line| parse_line_from(line).into_iter())
        .inspect(|parent| debug!("found parent: {}", &parent))
        .collect::<HashSet<_>>()
}

fn parse_line_from(line: &str) -> Result<Option<Parent>, String> {
    match FROM_RE.captures(line) {
        Some(matches) => {
            let tag_str = &matches[2];
            let tag_pattern = tag_to_re(tag_str)?;

            Ok(Some(Parent::new(&matches[1], tag_pattern, tag, &matches[3])))
        },
        None => {
            if line.contains(":") {
                warn!("warning: FROM line, but could not recognize version:\n  {}", line);
            } else {
                info!("skipping line because there is no version: {}", line);
            }
            Ok(None)
        }
    }
}

//TODO @mark: test
fn tag_to_re(tag_str: &str) -> Result<Regex, String> {
    let tag_escaped_for_re = &tag_str.replace("-", r"\-");
    let tag_digits_replaced = TAG_DIGITS_RE.replace_all(tag_escaped_for_re, "([0-9]+)").as_ref();
    let regex = Regex::new(tag_digits_replaced)
        .map_err(|err| format!("tag could not be turned into regex pattern; tag: {}, err: {}", tag_str, err));
    Ok(regex)
}