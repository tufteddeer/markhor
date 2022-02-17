use std::path::Path;
use std::thread;

use clap::Parser;
use log::{error, info};
use simple_logger::SimpleLogger;

#[cfg(feature = "serve")]
use yanos::serve::serve_files;
use yanos::{copy_static_files, generate_site, watch_directories};

const POSTS_DIR: &str = "posts";
const OUT_DIR: &str = "out";
const STATIC_DIR: &str = "static";
const TEMPLATES_DIR: &str = "templates";
const TEMPLATES_GLOB: &str = "templates/**/*";

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// Serve generated files
    #[clap(long)]
    serve: bool,
    /// Watch source directories for changes and rebuild
    #[clap(long)]
    watch: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    SimpleLogger::new()
        .with_module_level("globset", log::LevelFilter::Error)
        .init()?;

    let posts_dir = Path::new(POSTS_DIR);
    let output_dir = Path::new(OUT_DIR);

    generate_site(TEMPLATES_GLOB, posts_dir, output_dir)?;

    copy_static_files(STATIC_DIR, OUT_DIR)?;

    if args.serve {
        #[cfg(feature = "serve")]
        thread::spawn(|| {
            if let Err(error) = serve_files("127.0.0.1:8080", output_dir) {
                error!("Failed serving files: {}", error);
            }
        });
        #[cfg(not(feature = "serve"))]
        log::error!("Feature 'serve' was not enabled during compilation. Server not available");
    }
    if args.watch {
        #[cfg(feature = "watch")]
        println!("watching...");
        #[cfg(not(feature = "watch"))]
        log::error!("Feature 'watch' was not enabled during compilation. Watching not available");
    }

    if let Err(e) = watch_directories(TEMPLATES_DIR, POSTS_DIR, STATIC_DIR, |_| {
        info!("Change detected, regenerating...");
        if let Err(error) = generate_site(TEMPLATES_GLOB, POSTS_DIR, OUT_DIR) {
            error!("Failed generating site: {}", error);
        }
    }) {
        error!("Failed to watch files: {:?}", e)
    }
    Ok(())
}
