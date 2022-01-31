use std::{fs, path::Path};

use log::info;
use rust_templating::{render_markdown, render_markdown_into_template, write_output};
use simple_logger::SimpleLogger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_module_level("globset", log::LevelFilter::Error)
        .init()?;

    let posts_dir = Path::new("markdown");
    let output_dir = Path::new("out");

    info!("Using markdown files in {:?}", posts_dir);
    for entry in fs::read_dir(posts_dir)? {
        let name = entry?.file_name();
        let filepath = posts_dir.join(&name);

        let mut out_name = name.to_owned();
        out_name.push(".html");

        info!("Rendering {:?} to {:?}", name, out_name);
        let markdown_html = render_markdown(filepath.as_path())?;

        let result_html = render_markdown_into_template(markdown_html)?;

        write_output(output_dir, out_name, result_html)?;
    }

    Ok(())
}
