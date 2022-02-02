use std::{ops::Sub, path::Path, time::Instant};

use fs_extra::{copy_items, dir};
use log::info;
use rust_templating::{
    markdown::convert_posts,
    templating::{self, render_index},
    write_output,
};
use simple_logger::SimpleLogger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_module_level("globset", log::LevelFilter::Error)
        .init()?;

    let posts_dir = Path::new("markdown");
    let output_dir = Path::new("out");

    let start_time = Instant::now();

    let tera = templating::init_tera("templates/**/*.html");

    let post_metadata = convert_posts(&tera, posts_dir, output_dir)?;

    let index_html = render_index(&tera, &post_metadata)?;

    write_output(output_dir, "index.html", index_html)?;

    let elapsed_time = Instant::now().sub(start_time);
    log::info!("Took {}ms", &elapsed_time.as_millis());

    info!("Copying static assets");

    let options = dir::CopyOptions::new();
    let from = vec!["static"];
    copy_items(&from, "out", &options)?;

    Ok(())
}
