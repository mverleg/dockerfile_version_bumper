use ::std::cmp::Ordering;
use ::std::fmt;
use ::std::hash;
use ::std::hash::Hasher;
use ::std::path::PathBuf;

use ::derive_getters::Getters;
use ::derive_new::new;
use ::regex::Regex;

#[derive(Debug, Getters, new)]
pub struct Dockerfile {
    path: PathBuf,
    content: String,
}

#[derive(Debug, Getters, new)]
pub struct Parent {
    name: String,
    tag_pattern: Regex,
    tag: Tag,
    suffix: String,
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
        self.name == other.name && self.tag == other.tag
    }
}

impl Eq for Parent {}

impl hash::Hash for Parent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes());
        state.write(self.tag_pattern.as_str().as_bytes());
        self.tag.hash(state);
    }
}

#[derive(Debug, Getters, new)]
pub struct Tag {
    nrs: (u32, u32, u32, u32,),
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.nrs.0, self.nrs.1, self.nrs.2, self.nrs.3)
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

//TODO @mark: test
pub fn parse_tag(tag_pattern: &Regex, tag: &str) -> Result<Tag, String> {
    let tag = tag_pattern.find(tag)
        .ok_or_else(|| format!("could not extract digits from tag; tag: {}, pattern: {}", tag, tag_pattern.as_str()))?;
    tag;

    unimplemented!()
}
