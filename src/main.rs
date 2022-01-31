use std::{fs, path::Path};

use rust_templating::{render_markdown, render_markdown_into_template, write_output};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let posts_dir = Path::new("markdown");
    let output_dir = Path::new("out");

    println!("using posts in {:?}", posts_dir);
    for entry in fs::read_dir(posts_dir)? {
        let name = entry?.file_name();
        let filepath = posts_dir.join(&name);

        println!("rendering md file {:?}", filepath.as_path().as_os_str());
        let markdown_html = render_markdown(filepath.as_path())?;

        let result_html = render_markdown_into_template(markdown_html)?;

        let mut out_name = name.to_owned();
        out_name.push(".html");

        write_output(output_dir, out_name, result_html)?;
    }

    Ok(())
}
