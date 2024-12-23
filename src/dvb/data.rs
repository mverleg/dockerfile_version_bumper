use ::std::cmp::Ordering;
use ::std::fmt;
use ::std::hash;
use ::std::hash::Hasher;
use ::std::path::PathBuf;
use ::std::rc::Rc;

use ::derive_getters::Getters;
use ::derive_new::new;
use ::regex::Regex;

#[derive(Debug, Getters, new)]
pub struct Dockerfile {
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
    image_name: String,
    tag_pattern: Regex,
    tag: Tag,
    suffix: String,
}

impl Parent {
    pub fn explode(self) -> (PathBuf, String, Tag) {
        let Parent {
            dockerfile,
            image_name: name,
            tag,
            ..
        } = self;
        (dockerfile.path().to_owned(), name, tag)
    }
}

impl fmt::Display for Parent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.suffix.is_empty() {
            write!(
                f,
                "{}:{}@{}",
                &self.image_name,
                &self.tag,
                self.tag_pattern.as_str()
            )
        } else {
            write!(
                f,
                "{}:{}@{} {}",
                &self.image_name,
                &self.tag,
                self.tag_pattern.as_str(),
                &self.suffix
            )
        }
    }
}

impl PartialEq for Parent {
    fn eq(&self, other: &Self) -> bool {
        self.dockerfile.path() == other.dockerfile.path()
            && self.image_name == other.image_name
            && self.tag == other.tag
    }
}

impl Eq for Parent {}

impl hash::Hash for Parent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dockerfile.path().hash(state);
        state.write(self.image_name.as_bytes());
        state.write(self.tag_pattern.as_str().as_bytes());
        self.tag.hash(state);
    }
}

#[derive(Debug, Clone, Getters, new)]
pub struct Tag {
    name: String,
    nrs: (u32, u32, u32, u32),
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
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
