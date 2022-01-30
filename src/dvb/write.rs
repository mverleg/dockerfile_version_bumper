use ::std::collections::HashMap;
use ::std::fs;
use ::std::path::PathBuf;

use ::indexmap::IndexMap;

use crate::dvb::data::Tag;
use crate::Parent;

pub async fn update_all_dockerfiles(latest_tags: &IndexMap<Parent, Tag>) -> Result<(), String> {
    let new_content = updated_dockerfiles_content(latest_tags);
    unimplemented!()  //TODO @mark: TEMPORARY! REMOVE THIS!
}

fn updated_dockerfiles_content(latest_tags: &IndexMap<Parent, Tag>) -> IndexMap<PathBuf, String> {
    //TODO @mark: what if multiple updates to same Dockerfile?
    let mut files: HashMap<&PathBuf, String> = HashMap::new();
    for (parent, new_tag) in latest_tags.iter() {
        let content: &mut String = files.entry(parent.dockerfile().path())
            .or_insert_with(|| parent.dockerfile().content().to_owned());
        let q: String = parent.tag_pattern().replace_all(content, format!("{}", new_tag)).into_owned();
        *content = q;
        // let content = format!("# updated: {} {} -> {}!\n{}", parent.name(), parent.tag(), new_tag, parent.dockerfile().content());
        // fs::write(parent.dockerfile().path(), content)
        //     .map_err(|_| format!("failed to update Dockerfile '{}'", parent.name()))?
    }
    unimplemented!()  //TODO @mark: TEMPORARY! REMOVE THIS!
}

#[cfg(test)]
mod tests {
    use ::std::rc::Rc;

    use ::indexmap::indexmap;

    use crate::dvb::data::{Dockerfile, parse_tag};
    use crate::dvb::read::tag_to_re;

    use super::*;

    #[test]
    fn single() {
        let tag_str = "1.2.4-alpha";
        let dockerfile = Rc::new(Dockerfile::new(
            PathBuf::from("/fake/Dockerfile"),
            format!("namespace/image:{} AS build\n", &tag_str)));
        let tag_pattern = tag_to_re(&tag_str).unwrap();
        let tag = parse_tag(&tag_pattern, tag_str).unwrap();
        let parent = Parent::new(
            dockerfile,
            "namespace/image".to_owned(),
            tag_pattern,
            tag,
            "AS build".to_owned(),
        );
        let tags = updated_dockerfiles_content(&indexmap![
            parent => Tag::new(tag_str.to_owned(), (1, 3, 2, 0)),
        ]);
    }
}
