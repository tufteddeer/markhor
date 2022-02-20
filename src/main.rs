use std::path::Path;
use std::thread;

use clap::Parser;
use log::info;
use simple_logger::SimpleLogger;

use yanos::watch::watch_directories;
use yanos::{copy_static_files, generate_site};

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

    let serve_handle = if args.serve {
        Some(thread::spawn(move || {
            yanos::serve::serve_files("127.0.0.1:8080", &output_dir)
                .expect("Failed to start fileserver");
        }))
    } else {
        None
    };

    let watch_handle = if args.watch {
        info!("Watching files for changes...");

        let change_listener = |_| {
            log::info!("Change detected, regenerating...");
            if let Err(error) = generate_site(TEMPLATES_GLOB, POSTS_DIR, OUT_DIR) {
                log::error!("Failed generating site: {}", error);
            }

            if let Err(error) = copy_static_files(STATIC_DIR, OUT_DIR) {
                log::error!("Failed to copy static assets: {}", error);
            }
        };

        Some(thread::spawn(move || {
            watch_directories(TEMPLATES_DIR, POSTS_DIR, STATIC_DIR, change_listener)
                .expect("Failed to watch files");
        }))
    } else {
        None
    };

    if let Some(handle) = serve_handle {
        handle.join().expect("Failed to join serve thread");
    }
    if let Some(handle) = watch_handle {
        handle.join().expect("Failed to join watch thread");
    }
    Ok(())
}
