use std::{fs, io, ops::Sub, path::Path, time::Instant};

use fs_extra::{copy_items, dir};
use log::info;
use simple_logger::SimpleLogger;
use yanos::{
    compare_header_date, compare_option,
    markdown::convert_posts,
    templating::{self, render_index},
    write_output,
};

const POSTS_DIR: &str = "posts";
const OUT_DIR: &str = "out";
const STATIC_DIR: &str = "static";
const TEMPLATES_GLOB: &str = "templates/**/*";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_module_level("globset", log::LevelFilter::Error)
        .init()?;

    let posts_dir = Path::new(POSTS_DIR);
    let output_dir = Path::new(OUT_DIR);

    let start_time = Instant::now();

    let tera = templating::init_tera(TEMPLATES_GLOB);

    let mut post_metadata = convert_posts(&tera, posts_dir, output_dir)?;

    post_metadata.sort_unstable_by(|a, b| {
        compare_option(&b.header, &a.header, |meta_a, meta_b| {
            compare_header_date(meta_a, meta_b)
        })
    });

    let index_html = render_index(&tera, &post_metadata)?;

    write_output(output_dir, "index.html", index_html)?;

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

    Ok(())
}
