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

pub async fn bump_dockerfiles(
    dockerfiles: &[PathBuf],
    allow_parents: &[String],
    bump_major: bool,
    print: bool,
) -> Result<(), String> {
    assert!(bump_major, "bumping only minor versions not implemented, use --major");
    assert!(print, "in-place update not implemented, use --print");
    let dockerfiles = read_all_dockerfiles(dockerfiles).await?;
    let all_parents = extract_parents(&dockerfiles);
    let parents = filter_parents(all_parents, allow_parents)?;
    //https://registry.hub.docker.com/v1/repositories/${img}/tags
    unimplemented!()
}

#[derive(Debug)]
struct Dockerfile {
    path: PathBuf,
    content: String,
}

async fn read_all_dockerfiles(dockerfiles: &[PathBuf]) -> Result<Vec<Dockerfile>, String> {
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

fn filter_parents(all_parents: HashSet<Parent>, allow_parent_names: &[String]) -> Result<HashSet<Parent>, String> {
    if allow_parent_names.is_empty() {
        if all_parents.is_empty() {
            return Err("No FROM tags with versions were found in the Dockerfile(s)".to_owned())
        }
        return Ok(all_parents)
    }
    let allow_parent_names: HashSet<String> = HashSet::from_iter(allow_parent_names.iter().cloned());
    let parents = all_parents.into_iter()
        .filter(|parent| allow_parent_names.contains(&parent.name))
        .inspect(|parent| debug!("including parent (-p): {}", parent))
        .collect::<HashSet<_>>();
    if parents.is_empty() {
        return Err("None of the FROM tags given with --parent/-p were found in the Dockerfile(s)".to_owned())
    }
    Ok(parents)
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
