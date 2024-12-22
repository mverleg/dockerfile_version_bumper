use ::std::path::PathBuf;
use ::std::process::exit;
use ::std::time::SystemTime;

use ::derive_getters::Getters;
use ::env_logger;
use ::tokio;
use ::clap::Parser;
use ::dockerfile_version_bumper::bump_dockerfiles;
use ::dockerfile_version_bumper::TagUp;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(Debug, Parser, Getters)]
#[structopt(
    name = "dockerfile_version_bumper",
    about = "Simple tool for scanning `Dockerfile`s and switching the image tags that appear in `FROM` to their latest version."
)]
/// CLI arguments. Readme will be updated by release build.
pub struct Args {
    #[clap(
        long = "dockerfile",
        short = 'f',
        default_value = "Dockerfile",
    )]
    dockerfiles: Vec<PathBuf>,
    /// Parent images (FROM lines) base names that should be bumped. If empty, bumps every image in the Dockerfile that is found in the registry.
    #[clap(
        long = "parent",
        short = 'p',
    )]
    parents: Vec<String>,
    /// Allow bumping to new major versions (which might be incompatible), which is interpreted as the leading number in the version.
    #[clap(
        long = "major",
    )]
    bump_major: bool,
    /// Print the output instead of updating in-place (dry run).
    #[clap(
        long = "dry-run",
    )]
    dry_run: bool,
    /// Print version bumps in json format. Still bumps Dockerfiles unless --dry-run is also given.
    #[clap(
        long = "json",
    )]
    json: bool,
}

#[tokio::main]
async fn main() {
    let start = SystemTime::now();
    env_logger::init();
    let args = Args::parse();
    match bump_dockerfiles(
        args.dockerfiles(),
        args.parents(),
        *args.bump_major(),
        *args.dry_run(),
    )
    .await
    {
        Ok(latest_tags) => {
            if *args.json() {
                print_tags_json(&latest_tags);
            } else {
                print_tags_text(&latest_tags);
            }
            eprintln!(
                "finished in {} ms",
                SystemTime::now().duration_since(start).unwrap().as_millis()
            )
        }
        Err(err) => {
            eprintln!("Fatal! {}", err);
            exit(1);
        }
    }
}

fn print_tags_json(parent_latest_tags: &[TagUp]) {
    let mut is_first = true;
    println!("[");
    for up in parent_latest_tags {
        if is_first {
            is_first = false
        } else {
            println!(",");
        }
        print!("  {{\"image\": \"{}\", ", &up.image);
        print!("\"dockerfile\": \"{}\", ", up.dockerfile.to_string_lossy());
        print!("\"current_tag\": \"{}\", ", up.old_tag);
        print!("\"updated_tag\": \"{}\", ", up.new_tag);
        print!("\"is_update\": {}}}", up.old_tag != up.new_tag);
    }
    println!("\n]");
}

fn print_tags_text(parent_latest_tags: &[TagUp]) {
    for up in parent_latest_tags {
        if up.old_tag == up.new_tag {
            println!("{}\t{} (up-to-date)", up.image, up.old_tag)
        } else {
            println!("{}\t{} -> {}", up.image, up.old_tag, up.new_tag)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use indexmap::indexmap;
    use super::*;

    // Dockerfile version bumper fails with this error:
    // "Fatal! could not find the version 20221230-6970e9da nor any higher ones"
    // When the image contain a URL, like
    // "FROM docker-registry:8080/mverleg/dev-base:20221230-6970e9da"
    //TODO @mark: ^

    #[test]
    fn with_repository_url() {
        

        //TODO @mark:
        let image = "docker-registry:8080/namespace/image".to_owned();
        let path = PathBuf::from("/fake/Dockerfile");
        let tag_str = "1.2.4-alpha";
        let dockerfile = Rc::new(Dockerfile::new(
            path.clone(),
            format!("FROM {}:{} AS build\n", &image, &tag_str),
        ));

        bump_dockerfiles(

        );

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
        ])
            .unwrap();
        assert_eq!(
            tags,
            indexmap![
                path => format!("FROM docker-registry:8080/namespace/image:{} AS build\n", &tag_new),
            ]
        );
    }
}
