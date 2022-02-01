use log::{error, info};
use pulldown_cmark::{html, Parser};

use serde::Serialize;
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

#[derive(Debug, Serialize)]
pub struct PostMeta {
    pub source_file: String,
    pub rendered_to: String,
}

pub fn render_index(posts_meta: &[PostMeta]) -> Result<String, tera::Error> {
    let mut context = Context::new();

    context.insert("post_toc", &posts_meta);

    TERA_TEMPLATE.render("index.html", &context)
}

pub fn render_markdown(filepath: &Path) -> Result<String, Box<dyn Error>> {
    let input = fs::read_to_string(filepath)?;

    let parser = Parser::new(&input);

    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);

    Ok(html_out)
}

pub fn convert_posts(posts_dir: &Path, out_dir: &Path) -> Result<Vec<PostMeta>, Box<dyn Error>> {
    let mut post_metadata = Vec::<PostMeta>::new();

    info!("Using markdown files in {:?}", posts_dir);
    for entry in fs::read_dir(posts_dir)? {
        let name = entry?.file_name();
        let filepath = posts_dir.join(&name);

        let mut out_name = name.to_owned();
        out_name.push(".html");

        info!("Rendering {:?} to {:?}", name, out_name);
        let markdown_html = render_markdown(filepath.as_path())?;

        let result_html = render_markdown_into_template(markdown_html)?;

        write_output(out_dir, &out_name, result_html)?;

        post_metadata.push(PostMeta {
            source_file: name.into_string().unwrap(),
            rendered_to: out_name.into_string().unwrap(),
        })
    }

    Ok(post_metadata)
}

pub fn write_output(
    out_dir: &Path,
    filename: &OsString,
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
