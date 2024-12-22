use ::std::path::PathBuf;
use ::std::process::exit;
use ::std::time::SystemTime;

use ::clap::Parser;
use ::derive_getters::Getters;
use ::dockerfile_version_bumper::bump_dockerfiles;
use ::dockerfile_version_bumper::TagUp;
use ::env_logger;
use ::tokio;

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
    use super::*;
    use ::std::fs;
    use ::tempfile::NamedTempFile;

    #[tokio::test]
    async fn full_bump_with_repo() {
        let temp_file = NamedTempFile::new().unwrap();
        let image = "python".to_owned();
        let tag_str = "3.11";
        fs::write(&temp_file, format!("FROM {}:{} AS build\n", &image, &tag_str)).unwrap();
        bump_dockerfiles(
            &[temp_file.path().to_path_buf()],
            &[],
            false,
            false,
        ).await.unwrap();
        let content = fs::read_to_string(temp_file.path()).unwrap();
        dbg!(&content);
        assert!(*content >= *"FROM python:3.13");
        assert!(content.ends_with(" AS build\n"))
    }
}
