use ::std::path::PathBuf;
use std::fs::write;

use ::derive_new::new;
use ::indexmap::IndexMap;
use ::log::debug;

use crate::dvb::convert::image_tag_to_re;
use crate::dvb::data::Tag;
use crate::Parent;

pub async fn update_all_dockerfiles(latest_tags: &IndexMap<Parent, Tag>, dry_run: bool) -> Result<(), String> {
    let new_content = updated_dockerfiles_content(latest_tags)?;
    if dry_run {
        for (pth, content) in new_content {
            debug!("If this were not running in dry-mode, the Dockerfile '{}' would become:\n{}", pth.to_string_lossy(), content);
        }
        return Ok(());
    }
    write_dockerfiles(new_content)
}

async fn write_dockerfiles(path_contents: &IndexMap<PathBuf, String>) -> Result<(), String> {
    let mut futures = vec![];
    for (pth, content) in path_contents {
        debug!("writing updated Dockerfile to '{}'", pth.to_string_lossy());
        write(pth, content)
            .map_err(|err| format!("failed to write updated Dockerfile: {}", err))?;
    }
    let mut futures = vec![];
    for path in dockerfiles {
        futures.push(read_dockerfile(path));
    }
    Ok(try_join_all(futures).await?.into_iter()
        .map(|df| Rc::new(df))
        .collect::<Vec<_>>())
}

Ok(())
}

fn updated_dockerfiles_content(
    latest_tags: &IndexMap<Parent, Tag>,
) -> Result<IndexMap<PathBuf, String>, String> {
    let mut files: IndexMap<PathBuf, String> = IndexMap::new();
    for (parent, new_tag) in latest_tags.iter() {
        let content: &mut String = files
            .entry(parent.dockerfile().path().to_owned())
            .or_insert_with(|| parent.dockerfile().content().to_owned());
        let image_pattern =
            image_tag_to_re(parent.image_name(), parent.tag().name(), parent.suffix())?;
        let new_image = format!("FROM {}:{}{}", parent.image_name(), new_tag, parent.suffix());
        debug_assert!(
            image_pattern.is_match(content),
            "did not find image tag in dockerfile"
        );
        *content = image_pattern
            .replace_all(content, new_image)
            .into_owned();
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use ::std::rc::Rc;

    use ::indexmap::indexmap;

    use crate::dvb::convert::{parse_tag, tag_to_re};
    use crate::dvb::data::Dockerfile;

    use super::*;

    #[test]
    fn re_replace() {
        let res = image_tag_to_re("namespace/image", "1.2.8-alpha", " AS build").unwrap()
            .replace_all("FROM  namespace/image:1.2.8-alpha  AS build\n",
                         "FROM namespace/image:1.3.2-alpha AS build".to_owned());
        assert_eq!("FROM namespace/image:1.3.2-alpha AS build\n", res);
    }

    #[test]
    fn single() {
        let image = "namespace/image".to_owned();
        let path = PathBuf::from("/fake/Dockerfile");
        let tag_str = "1.2.4-alpha";
        let dockerfile = Rc::new(Dockerfile::new(
            path.clone(),
            format!("FROM {}:{} AS build\n", &image, &tag_str),
        ));
        let tag_pattern = tag_to_re(tag_str).unwrap();
        let tag_old = parse_tag(&tag_pattern, tag_str).unwrap();
        let tag_new = Tag::new("1.3.2-alpha".to_owned(), (1, 3, 2, 0));

        let parent = Parent::new(
            dockerfile,
            image,
            tag_pattern,
            tag_old,
            " AS build".to_owned(),
        );

        let tags = updated_dockerfiles_content(&indexmap![
            parent => tag_new.clone(),
        ]).unwrap();
        assert_eq!(
            tags,
            indexmap![
                path => format!("FROM namespace/image:{} AS build\n", &tag_new),
            ]
        );
    }

    #[test]
    fn multi_file_multi_tag() {
        let image1 = "namespace/image".to_owned();
        let tag1_str = "1.2.4-alpha";
        let tag1_pattern = tag_to_re(tag1_str).unwrap();
        let tag_old1 = parse_tag(&tag1_pattern, tag1_str).unwrap();
        let tag_new1 = Tag::new("1.3.2-alpha".to_owned(), (1, 3, 2, 0));

        let image2 = "namespace/image2".to_owned();
        let tag2_str = "0.3.7-rc2";
        let tag2_pattern = tag_to_re(tag2_str).unwrap();
        let tag_old2 = parse_tag(&tag2_pattern, tag2_str).unwrap();
        let tag_new2 = Tag::new("0.4.4-rc1".to_owned(), (0, 4, 4, 1));

        let path1 = PathBuf::from("/fake/one/Dockerfile");
        let dockerfile_a = Rc::new(Dockerfile::new(
            path1.clone(),
            format!(
                "FROM {}:{} AS build\n\
                    FROM {}:{}\n\
                    RUN echo done",
                &image1, &tag1_str, &image2, &tag2_str
            ),
        ));
        let path2 = PathBuf::from("/fake/two/Dockerfile");
        let dockerfile_b = Rc::new(Dockerfile::new(
            path2.clone(),
            format!("FROM {}:{} AS pre\n", &image1, &tag1_str),
        ));

        let parent_a1 = Parent::new(
            dockerfile_a.clone(),
            image1.clone(),
            tag1_pattern.clone(),
            tag_old1.clone(),
            " AS build".to_owned(),
        );
        let parent_a2 = Parent::new(dockerfile_a, image2, tag2_pattern, tag_old2, "".to_owned());
        let parent_b1 = Parent::new(
            dockerfile_b,
            "namespace/image".to_owned(),
            tag1_pattern,
            tag_old1,
            " AS pre".to_owned(),
        );

        let tags = updated_dockerfiles_content(&indexmap![
            parent_a1 => tag_new1.clone(),
            parent_a2 => tag_new2.clone(),
            parent_b1 => tag_new1.clone(),
        ]).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[&path1], "FROM namespace/image:1.3.2-alpha AS build\nFROM namespace/image2:0.4.4-rc1\nRUN echo done");
        assert_eq!(tags[&path2], "FROM namespace/image:1.3.2-alpha AS pre\n");
    }

    #[test]
    fn do_not_match_in_run_cmd() {
        let image = "namespace/image".to_owned();
        let path = PathBuf::from("/fake/Dockerfile");
        let tag_str = "1.2.4-alpha";
        let dockerfile = Rc::new(Dockerfile::new(
            path.clone(),
            format!(
                "FROM {}:{} AS build\n\
            RUN echo 'Using {}:{} AS build '\n",
                &image, &tag_str, &image, &tag_str
            ),
        ));
        let tag_pattern = tag_to_re(tag_str).unwrap();
        let tag_old = parse_tag(&tag_pattern, tag_str).unwrap();
        let tag_new = Tag::new("1.3.2-alpha".to_owned(), (1, 3, 2, 0));

        let parent = Parent::new(
            dockerfile,
            image,
            tag_pattern,
            tag_old,
            " AS build".to_owned(),
        );

        let tags = updated_dockerfiles_content(&indexmap![
            parent => tag_new.clone(),
        ]).unwrap();
        assert_eq!(
            tags,
            indexmap![
                path => format!("FROM namespace/image:{} AS build\nRUN echo \
                'Using namespace/image:{} AS build '\n", &tag_new, &tag_str),
            ]
        );
    }
}
