use ::std::collections::HashSet;
use ::std::fmt;
use ::std::fs::read_to_string;
use ::std::hash;
use ::std::path::Path;
use ::std::path::PathBuf;

use ::lazy_static::lazy_static;
use ::log::{debug, info, warn};
use ::regex::Regex;

lazy_static! {
    static ref FROM_RE: Regex = Regex::new(r"^FROM\s+(\S+):(\S+)\s*(.*)$").unwrap();
}

pub fn bump_dockerfiles(
    dockerfiles: &[PathBuf],
    parents: &[String],
    bump_major: bool,
    print: bool,
) -> Result<(), String> {
    assert!(bump_major, "bumping only minor versions not implemented, use --major");
    assert!(print, "in-place update not implemented, use --print");
    let dockerfiles = read_all_dockerfiles(dockerfiles)?;
    let parents = extract_parents(&dockerfiles);
    unimplemented!()
}

#[derive(Debug)]
struct Dockerfile {
    path: PathBuf,
    content: String,
}

fn read_all_dockerfiles(dockerfiles: &[PathBuf]) -> Result<Vec<Dockerfile>, String> {
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

#[derive(Debug, Eq)]
struct Parent {
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

fn extract_parents(dockerfiles: &[Dockerfile]) -> HashSet<Parent> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_from_version_as() {
        let parent = parse_line_from("FROM node:lts-alpine3.14 AS editor").unwrap();
        assert_eq!(parent, Parent::from(("node", "lts-alpine3.14", "AS editor")))
    }
}
