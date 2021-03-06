use ::std::collections::HashSet;
use ::std::path::Path;
use ::std::path::PathBuf;
use ::std::rc::Rc;

use ::futures::future::try_join_all;
use ::lazy_static::lazy_static;
use ::log::{info, warn};
use ::regex::Regex;
use ::tokio::fs::read_to_string;

use crate::dvb::convert::{parse_tag, tag_to_re};
use crate::Parent;

use super::data::Dockerfile;

lazy_static! {
    static ref FROM_RE: Regex = Regex::new(r"^FROM\s+(\S+):(\S+)\s*( .*)?$").unwrap();
}

pub async fn read_all_dockerfiles(dockerfiles: &[PathBuf]) -> Result<Vec<Rc<Dockerfile>>, String> {
    let mut futures = vec![];
    for path in dockerfiles {
        futures.push(read_dockerfile(path));
    }
    Ok(try_join_all(futures)
        .await?
        .into_iter()
        .map(Rc::new)
        .collect::<Vec<_>>())
}

async fn read_dockerfile(path: &Path) -> Result<Dockerfile, String> {
    info!("reading dockerfile: {}", path.to_string_lossy());
    match read_to_string(path).await {
        Ok(content) => Ok(Dockerfile::new(path.to_path_buf(), content)),
        Err(err) => Err(format!(
            "Could not read Dockerfile '{}'.\n\
            Provide a correct path using -f PATH.\nError: {}",
            path.to_string_lossy(),
            err
        )),
    }
}

pub fn extract_parents(dockerfiles: &[Rc<Dockerfile>]) -> Result<HashSet<Parent>, String> {
    dockerfiles
        .iter()
        .flat_map(|file| file.content().lines().map(|line| (file.clone(), line)))
        .filter(|(_, line)| line.starts_with("FROM "))
        .map(|(file, line)| parse_line_from(file, line))
        .flat_map(|res_opt| res_opt.transpose().into_iter())
        //.inspect(|parent| debug!("found parent: {}", &parent))
        .collect()
}

fn parse_line_from(dockerfile: Rc<Dockerfile>, line: &str) -> Result<Option<Parent>, String> {
    match FROM_RE.captures(line) {
        Some(matches) => {
            let name = matches[1].to_owned();
            let tag_str = &matches[2];
            let tag_pattern = tag_to_re(tag_str)?;
            let tag = parse_tag(&tag_pattern, tag_str)?;
            let suffix = matches
                .get(3)
                .map(|s| s.as_str())
                .unwrap_or_else(|| "")
                .to_owned();
            Ok(Some(Parent::new(
                dockerfile,
                name,
                tag_pattern,
                tag,
                suffix,
            )))
        }
        None => {
            if line.contains(':') {
                warn!(
                    "warning: FROM line, but could not recognize version:\n  {}",
                    line
                );
            } else {
                info!("skipping line because there is no version: {}", line);
            }
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use ::regex::Regex;

    use crate::dvb::data::Tag;

    use super::*;

    #[test]
    fn parse_from_version_date() {
        let dockerfile = Rc::new(Dockerfile::new(PathBuf::from("file.ext"), "".to_owned()));
        let parent = parse_line_from(
            dockerfile.clone(),
            "FROM mverleg/rust_nightly_musl_base:2021-10-17_11",
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            parent,
            Parent::new(
                dockerfile,
                "mverleg/rust_nightly_musl_base".to_owned(),
                Regex::new("").unwrap(),
                Tag::new("2021-10-17_11".to_owned(), (2021, 10, 17, 11)),
                "".to_owned()
            )
        );
        assert_eq!(
            parent.tag_pattern().as_str(),
            r"^([0-9]+)\-([0-9]+)\-([0-9]+)_([0-9]+)$"
        );
    }

    #[test]
    fn parse_from_version_as() {
        let dockerfile = Rc::new(Dockerfile::new(PathBuf::from("file.ext"), "".to_owned()));
        let parent = parse_line_from(dockerfile.clone(), "FROM node:lts-alpine3.14 AS editor")
            .unwrap()
            .unwrap();
        assert_eq!(
            parent,
            Parent::new(
                dockerfile,
                "node".to_owned(),
                Regex::new("").unwrap(),
                Tag::new("lts-alpine3.14".to_owned(), (3, 14, 0, 0)),
                "AS editor".to_owned()
            )
        );
        assert_eq!(
            parent.tag_pattern().as_str(),
            r"^lts\-alpine([0-9]+)\.([0-9]+)$"
        );
    }
}
