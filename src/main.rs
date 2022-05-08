use ::std::path::PathBuf;
use ::std::process::exit;
use ::std::time::SystemTime;

use ::derive_getters::Getters;
use ::env_logger;
use ::structopt::StructOpt;
use ::tokio;

use ::dockerfile_version_bumper::bump_dockerfiles;
use ::dockerfile_version_bumper::TagUp;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(Debug, StructOpt, Getters)]
#[structopt(
    name = "dockerfile_version_bumper",
    about = "Simple tool for scanning `Dockerfile`s and switching the image tags that appear in `FROM` to their latest version."
)]
/// CLI arguments. Readme will be updated by release build.
pub struct Args {
    #[structopt(
        long = "dockerfile",
        short = "f",
        default_value = "Dockerfile",
        parse(from_os_str)
    )]
    dockerfiles: Vec<PathBuf>,
    #[structopt(
        long = "parent",
        short = "p",
        help = "Parent images (FROM lines) base names that should be bumped. If empty, bumps every image in the Dockerfile that is found in the registry."
    )]
    parents: Vec<String>,
    #[structopt(
        long = "major",
        help = "Allow bumping to new major versions (which might be incompatible), which is interpreted as the leading number in the version."
    )]
    bump_major: bool,
    #[structopt(
        long = "dry-run",
        help = "Print the output instead of updating in-place (dry run)."
    )]
    dry_run: bool,
    #[structopt(
        long = "json",
        help = "Print version bumps in json format. Still bumps Dockerfiles unless --dry-run is also given."
    )]
    json: bool,
}

#[tokio::main]
async fn main() {
    let start = SystemTime::now();
    env_logger::init();
    let args = Args::from_args();
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
