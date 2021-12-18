use ::std::collections::HashSet;
use ::std::fs::read_to_string;
use ::std::path::Path;
use ::std::path::PathBuf;

use ::futures::{FutureExt, StreamExt, TryFutureExt, TryStreamExt};
use ::lazy_static::lazy_static;
use ::log::{debug, info, warn};
use ::regex::Regex;

use crate::dvb::data::parse_tag;
use crate::Parent;

use super::data::Dockerfile;

lazy_static! {
    static ref FROM_RE: Regex = Regex::new(r"^FROM\s+(\S+):(\S+)\s*(.*)$").unwrap();
    static ref TAG_DIGITS_RE: Regex = Regex::new(r"[0-9]+").unwrap();
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

pub fn extract_parents(dockerfiles: &[Dockerfile]) -> Result<HashSet<Parent>, String> {
    dockerfiles.iter()
        .flat_map(|file| file.content().lines())
        .filter(|line| line.starts_with("FROM "))
        .map(|line| parse_line_from(line))
        .flat_map(|res_opt| res_opt.transpose().into_iter())
        //.inspect(|parent| debug!("found parent: {}", &parent))
        .collect()
}

fn parse_line_from(line: &str) -> Result<Option<Parent>, String> {
    match FROM_RE.captures(line) {
        Some(matches) => {
            let name = matches[1].to_owned();
            let tag_str = &matches[2];
            let tag_pattern = tag_to_re(tag_str)?;
            let tag = parse_tag(&tag_pattern, tag_str)?;
            let suffix = matches[3].to_owned();
            Ok(Some(Parent::new(name, tag_pattern, tag, suffix)))
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
    dbg!(tag_str);  //TODO @mark: TEMPORARY! REMOVE THIS!
    let tag_digits_replaced = TAG_DIGITS_RE.replace_all(tag_escaped_for_re, "([0-9]+)");
    dbg!(tag_digits_replaced.as_ref());  //TODO @mark: TEMPORARY! REMOVE THIS!
    let regex = Regex::new(tag_digits_replaced.as_ref())
        .map_err(|err| format!("tag could not be turned into regex pattern; tag: {}, err: {}", tag_str, err))?;
    Ok(regex)
}

#[cfg(test)]
mod tests {
    use ::regex::Regex;

    use crate::dvb::data::Tag;

    use super::*;

    #[test]
    fn parse_from_version_date() {
        let parent = parse_line_from("FROM mverleg/rust_nightly_musl_base:2021-10-17_11").unwrap().unwrap();
        assert_eq!(parent, Parent::new("mverleg/rust_nightly_musl_base".to_owned(),
            Regex::new(r"2021-10-17_11").unwrap(), Tag::new((2021, 10, 17, 11)), "".to_owned()));
    }

    #[test]
    fn parse_from_version_as() {
        let parent = parse_line_from("FROM node:lts-alpine3.14 AS editor").unwrap().unwrap();
        assert_eq!(parent, Parent::new("node".to_owned(),
            Regex::new(r"lts-alpine3.14").unwrap(), Tag::new((3, 14, 0, 0)), "AS editor".to_owned()));
    }
}
