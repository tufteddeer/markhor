use std::{fs, io, ops::Sub, path::Path, time::Instant};

use clap::Parser;
use fs_extra::{copy_items, dir};
use log::info;
use simple_logger::SimpleLogger;

use yanos::generate_site;
#[cfg(feature = "serve")]
use yanos::serve::serve_files;

const POSTS_DIR: &str = "posts";
const OUT_DIR: &str = "out";
const STATIC_DIR: &str = "static";
const TEMPLATES_GLOB: &str = "templates/**/*";

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// Serve generated files
    #[clap(long)]
    serve: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    SimpleLogger::new()
        .with_module_level("globset", log::LevelFilter::Error)
        .init()?;

    let posts_dir = Path::new(POSTS_DIR);
    let output_dir = Path::new(OUT_DIR);

    let start_time = Instant::now();

    generate_site(TEMPLATES_GLOB, posts_dir, output_dir)?;

    let elapsed_time = Instant::now().sub(start_time);
    log::info!("Took {}ms", &elapsed_time.as_millis());

    if let Err(e) = fs::read_dir(STATIC_DIR) {
        match e.kind() {
            io::ErrorKind::NotFound => {
                info!("No static directory found, skipping");
            }
            _ => {
                panic!("Failed to access static directory {}: {}", STATIC_DIR, e);
            }
        }
    } else {
        info!("Copying static assets");

        let mut options = dir::CopyOptions::new();
        options.overwrite = true;

        let from = vec![STATIC_DIR];
        copy_items(&from, output_dir, &options)?;
    }

    if args.serve {
        #[cfg(feature = "serve")]
        serve_files("127.0.0.1:8080", output_dir)?;
        #[cfg(not(feature = "serve"))]
        log::error!("Feature 'serve' was not enabled during compilation. Server not available");
    }

    Ok(())
}
