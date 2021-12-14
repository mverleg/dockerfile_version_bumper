use std::process::exit;
use ::env_logger;
use ::structopt::StructOpt;
use dockerfile_version_bumper::bump_dockerfiles;

use crate::args::Args;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

mod args;

fn main() {
    env_logger::init();
    let args = Args::from_args();
    match bump_dockerfiles(&args.dockerfiles, &args.parents, args.bump_major, args.print) {
        Ok(()) => {
            eprintln!("done")
        }
        Err(err) => {
            eprintln!("Fatal! {}", err);
            exit(1);
        }
    }
}
