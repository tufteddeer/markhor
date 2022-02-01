use std::{ffi::OsString, ops::Sub, path::Path, time::Instant};

use rust_templating::{convert_posts, render_index, write_output};
use simple_logger::SimpleLogger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_module_level("globset", log::LevelFilter::Error)
        .init()?;

    let posts_dir = Path::new("markdown");
    let output_dir = Path::new("out");

    let start_time = Instant::now();

    let post_metadata = convert_posts(posts_dir, output_dir)?;

    let index_html = render_index(&post_metadata)?;

    write_output(output_dir, &OsString::from("index.html"), index_html)?;

    let elapsed_time = Instant::now().sub(start_time);
    log::info!("Took {}ms", &elapsed_time.as_millis());

    Ok(())
}
