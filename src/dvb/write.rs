use ::std::collections::HashMap;
use ::std::fs;
use ::std::path::PathBuf;

use ::indexmap::IndexMap;

use crate::dvb::data::Tag;
use crate::Parent;

pub async fn update_all_dockerfiles(latest_tags: &IndexMap<Parent, Tag>) -> Result<(), String> {
    let new_content = updated_dockerfiles_content(latest_tags);
    // fs::write(parent.dockerfile().path(), content)
    //     .map_err(|_| format!("failed to update Dockerfile '{}'", parent.name()))?
    unimplemented!()  //TODO @mark: TEMPORARY! REMOVE THIS!
}

fn updated_dockerfiles_content(latest_tags: &IndexMap<Parent, Tag>) -> IndexMap<PathBuf, String> {
    //TODO @mark: what if multiple updates to same Dockerfile?
    let mut files: IndexMap<PathBuf, String> = IndexMap::new();
    for (parent, new_tag) in latest_tags.iter() {
        let content: &mut String = files.entry(parent.dockerfile().path().to_owned())
            .or_insert_with(|| parent.dockerfile().content().to_owned());
        let q: String = parent.tag_pattern().replace_all(content, format!("{}", new_tag)).into_owned();
        *content = q;
        // let content = format!("# updated: {} {} -> {}!\n{}", parent.name(), parent.tag(), new_tag, parent.dockerfile().content());
    }
    files
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
        let path = PathBuf::from("/fake/Dockerfile");
        let tag_str = "1.2.4-alpha";
        let dockerfile = Rc::new(Dockerfile::new(
            path.clone(),
            format!("namespace/image:{} AS build\n", &tag_str)));
        let tag_pattern = tag_to_re(&tag_str).unwrap();
        let tag_old = parse_tag(&tag_pattern, tag_str).unwrap();
        let tag_new = Tag::new("1.3.2-alpha".to_owned(), (1, 3, 2, 0));

        let parent = Parent::new(
            dockerfile,
            "namespace/image".to_owned(),
            tag_pattern,
            tag_old,
            "AS build".to_owned(),
        );

        let tags = updated_dockerfiles_content(&indexmap![
            parent => tag_new.clone(),
        ]);
        assert_eq!(tags, indexmap![
            path.clone() => format!("namespace/image:{} AS build\n", &tag_new),
        ]);
    }

    #[test]
    fn multi_file_multi_tag() {
        let tag1_str = "1.2.4-alpha";
        let tag1_pattern = tag_to_re(&tag1_str).unwrap();
        let tag_old1 = parse_tag(&tag1_pattern, tag1_str).unwrap();
        let tag_new1 = Tag::new("1.3.2-alpha".to_owned(), (1, 3, 2, 0));

        let tag2_str = "0.3.7-rc2";
        let tag2_pattern = tag_to_re(&tag2_str).unwrap();
        let tag_old2 = parse_tag(&tag2_pattern, tag2_str).unwrap();
        let tag_new2 = Tag::new("0.4.4-rc1".to_owned(), (0, 4, 4, 1));

        let path1 = PathBuf::from("/fake/one/Dockerfile");
        let dockerfile_a = Rc::new(Dockerfile::new(
            path1.clone(),
            format!("namespace/image:{} AS build\n\
                    namespace/image2:{}\n",
                    &tag1_str, &tag2_str)));
        let path2 = PathBuf::from("/fake/two/Dockerfile");
        let dockerfile_b = Rc::new(Dockerfile::new(
            path2.clone(),
            format!("namespace/image:{} AS pre\n", &tag1_str)));

        let parent_a1 = Parent::new(
            dockerfile_a.clone(),
            "namespace/image".to_owned(),
            tag1_pattern.clone(),
            tag_old1.clone(),
            "AS build".to_owned(),
        );
        let parent_a2 = Parent::new(
            dockerfile_a,
            "namespace/image".to_owned(),
            tag2_pattern,
            tag_old2,
            "".to_owned(),
        );
        let parent_b1 = Parent::new(
            dockerfile_b,
            "namespace/image2".to_owned(),
            tag1_pattern,
            tag_old1,
            "AS pre".to_owned(),
        );

        let tags = updated_dockerfiles_content(&indexmap![
            parent_a1 => tag_new1.clone(),
            parent_a2 => tag_new2.clone(),
            parent_b1 => tag_new1.clone(),
        ]);
        assert_eq!(tags, indexmap![
            path1.clone() => format!("namespace/image:{} AS build\n\
                    namespace/image2:{}\n", &tag_new1, &tag_new2),
            path2.clone() => format!("namespace/image:{} AS build\n", &tag_new2),
        ]);
    }
}
