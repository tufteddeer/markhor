use log::{error, info};
use pulldown_cmark::{html, Parser};
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{
    error::Error,
    fs::{self, File},
    io,
};
use tera::{Context, Tera};
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref TERA_TEMPLATE: Tera = {
        info!("Creating Tera");
        let mut tera = match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to create tera: {}", e);
                std::process::exit(1);
            }
        };

        tera.autoescape_on(vec![]);

        tera
    };
}

pub fn render_markdown_into_template(markdown: String) -> Result<String, tera::Error> {
    let mut context = Context::new();

    context.insert("markdown_content", &markdown);

    TERA_TEMPLATE.render("post.html", &context)
}

pub fn render_markdown(filepath: &Path) -> Result<String, Box<dyn Error>> {
    let input = fs::read_to_string(filepath)?;

    let parser = Parser::new(&input);

    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);

    Ok(html_out)
}

pub fn write_output(
    out_dir: &Path,
    filename: OsString,
    content: String,
) -> Result<(), Box<dyn Error>> {
    if let Err(e) = fs::read_dir(out_dir) {
        match e.kind() {
            io::ErrorKind::NotFound => {
                info!("Creating output directory {:?}", out_dir);
                fs::create_dir(out_dir)?;
            }
            _ => {
                panic!("Failed to access output directory {:?}: {}", out_dir, e);
            }
        }
    };

    let mut filepath = PathBuf::from(out_dir);
    filepath.push(filename);
    let mut file = File::create(filepath)?;

    write!(file, "{}", content)?;

    Ok(())
}
