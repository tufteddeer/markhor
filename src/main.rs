use std::path::Path;
use std::thread::{self, JoinHandle};

use clap::Parser;
use log::info;
use simple_logger::SimpleLogger;

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
        spawn_fileserver(output_dir)
    } else {
        None
    };

    if args.watch {
        #[cfg(feature = "watch")]
        {
            info!("Watching files for changes...");
            if let Err(e) =
                yanos::watch::watch_directories(TEMPLATES_DIR, POSTS_DIR, STATIC_DIR, |_| {
                    log::info!("Change detected, regenerating...");
                    if let Err(error) = generate_site(TEMPLATES_GLOB, POSTS_DIR, OUT_DIR) {
                        log::error!("Failed generating site: {}", error);
                    }
                })
            {
                log::error!("Failed to watch files: {:?}", e)
            }
        }
        #[cfg(not(feature = "watch"))]
        log::error!("Feature 'watch' was not enabled during compilation. Watching not available");
    }

    if let Some(handle) = serve_handle {
        handle.join().expect("Failed to join threads");
    }
    Ok(())
}

#[cfg(feature = "serve")]
fn spawn_fileserver(output_dir: &'static Path) -> Option<JoinHandle<()>> {
    Some(thread::spawn(move || {
        if let Err(error) = yanos::serve::serve_files("127.0.0.1:8080", &output_dir) {
            log::error!("Failed serving files: {}", error);
        }
    }))
}
#[cfg(not(feature = "serve"))]
fn spawn_fileserver(_: &'static Path) -> Option<JoinHandle<()>> {
    log::error!("Feature 'serve' was not enabled during compilation. Server not available");
    None
}
