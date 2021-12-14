use ::env_logger;
use ::structopt::StructOpt;

use crate::args::Args;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

mod args;

fn main() {
    env_logger::init();
    let args = Args::from_args();
    println!("Hello, world! {:?}", args);
}
