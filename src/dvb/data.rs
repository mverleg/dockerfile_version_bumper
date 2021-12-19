use ::std::cmp::Ordering;
use ::std::fmt;
use ::std::hash;
use ::std::hash::Hasher;
use ::std::path::PathBuf;
use std::rc::Rc;

use ::derive_getters::Getters;
use ::derive_new::new;
use ::regex::Match;
use ::regex::Regex;

#[derive(Debug, Getters, new)]
pub struct Dockerfile {
    #[allow(dead_code)]  //TODO @mark: TEMPORARY! REMOVE THIS!
    path: PathBuf,
    content: String,
}

impl PartialEq for Dockerfile {
    fn eq(&self, other: &Self) -> bool {
        self.path() == other.path()
    }
}

impl Eq for Dockerfile {}

impl PartialOrd for Dockerfile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.path.partial_cmp(&other.path)
    }
}

impl Ord for Dockerfile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.cmp(&other.path)
    }
}

#[derive(Debug, Getters, new)]
pub struct Parent {
    dockerfile: Rc<Dockerfile>,
    name: String,
    tag_pattern: Regex,
    tag: Tag,
    suffix: String,
}

impl Parent {
    pub fn explode(self) -> (PathBuf, String, Tag) {
        let Parent {
            dockerfile,
            name,
            tag,
            ..
        } = self;
        (dockerfile.path().to_owned(), name, tag)
    }
}

impl fmt::Display for Parent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.suffix.is_empty() {
            write!(f, "{}:{}@{}", &self.name, &self.tag, self.tag_pattern.as_str())
        } else {
            write!(f, "{}:{}@{} {}", &self.name, &self.tag, self.tag_pattern.as_str(), &self.suffix)
        }
    }
}

impl PartialEq for Parent {
    fn eq(&self, other: &Self) -> bool {
        self.dockerfile.path() == other.dockerfile.path() && self.name == other.name && self.tag == other.tag
    }
}

impl Eq for Parent {}

impl hash::Hash for Parent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dockerfile.path().hash(state);
        state.write(self.name.as_bytes());
        state.write(self.tag_pattern.as_str().as_bytes());
        self.tag.hash(state);
    }
}

#[derive(Debug, Clone, Getters, new)]
pub struct Tag {
    name: String,
    nrs: (u32, u32, u32, u32,),
}

impl Tag {
    pub fn major(&self) -> u32 {
        self.nrs.0
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.nrs == other.nrs
    }
}

impl Eq for Tag {}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.nrs.partial_cmp(&other.nrs)
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.nrs.cmp(&other.nrs)
    }
}

impl hash::Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.nrs.0);
        state.write_u32(self.nrs.1);
        state.write_u32(self.nrs.2);
        state.write_u32(self.nrs.3);
    }
}

pub fn parse_tag(tag_pattern: &Regex, tag: impl Into<String>) -> Result<Tag, String> {
    let tag = tag.into();
    let parts = tag_pattern.captures(&tag)
        .ok_or_else(|| format!("could not extract digits from tag; tag: {}, pattern: {}, failed to capture", &tag, tag_pattern.as_str()))?;
    let nrs = (
        match_to_nr(parts.get(1)),
        match_to_nr(parts.get(2)),
        match_to_nr(parts.get(3)),
        match_to_nr(parts.get(4)),
    );
    Ok(Tag::new(tag, nrs))
}

fn match_to_nr(mtch: Option<Match>) -> u32 {
    mtch.map(|mtch| mtch.as_str().parse::<u32>().unwrap()).unwrap_or_else(|| 0)
}
