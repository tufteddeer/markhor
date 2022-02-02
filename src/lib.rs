use log::{error, info};
use pulldown_cmark::{html, Parser};

use serde::{Deserialize, Serialize};

use std::io::Write;
use std::path::{Path, PathBuf};
use std::{
    error::Error,
    fs::{self, File},
    io,
};
use tera::{Context, Tera};
use toml::value::Datetime;
#[macro_use]
extern crate lazy_static;

const MARKDOWN_HEADER_DELIMITER: &str = "---\n";

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

#[derive(Debug, Serialize, Deserialize)]
pub struct PostHeader {
    pub title: Option<String>,
    pub date: Option<Datetime>,
}

#[derive(Debug, Serialize)]
pub struct PostMeta {
    pub source_file: String,
    pub rendered_to: String,
    pub header: Option<PostHeader>,
}

pub fn render_index(posts_meta: &[PostMeta]) -> Result<String, tera::Error> {
    let mut context = Context::new();

    context.insert("post_toc", &posts_meta);

    TERA_TEMPLATE.render("index.html", &context)
}

pub fn render_markdown(markdown: &str) -> String {
    let parser = Parser::new(markdown);

    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);

    html_out
}

/// Parse an optional [PostHeader] located at the start of a markdown document (beginning and ending with [MARKDOWN_META_DELIMITER] )
///
/// # Examples
///
/// ```
/// use rust_templating::split_md_and_header;
/// use std::str::FromStr;
/// use toml::value::Datetime;
/// let input = r#"---
/// title = "mytitle"
/// ---
/// *bold*"#.to_string();
///
///
/// let (header, content) = split_md_and_header(&input).unwrap();
///
/// assert!(header.is_some());
/// let header = header.unwrap();
///
/// assert_eq!(header.title.unwrap(), "mytitle".to_string());
/// assert_eq!(content, "*bold*")
/// ```
pub fn split_md_and_header(input: &str) -> Result<(Option<PostHeader>, &str), toml::de::Error> {
    if !input.starts_with(MARKDOWN_HEADER_DELIMITER) {
        return Ok((None, input));
    }

    let mut parts = input.splitn(3, MARKDOWN_HEADER_DELIMITER);

    parts.next();

    let header = parts.next().unwrap();
    let content = parts.next().unwrap_or(header);

    println!("header: {}", header);
    println!("content: {}", content);

    if header == content {
        println!("{:?} has no meta information", "filepath");
        Ok((None, content))
    } else {
        println!("parsing {}", header);
        let header: PostHeader = toml::from_str(header)?;
        println!("{:?}", header);
        Ok((Some(header), content))
    }
}

pub fn convert_posts(
    posts_dir: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<Vec<PostMeta>, Box<dyn Error>> {
    let posts_dir = posts_dir.as_ref();
    let mut post_metadata = Vec::<PostMeta>::new();

    info!("Using markdown files in {:?}", posts_dir);
    for entry in fs::read_dir(posts_dir)? {
        let name = entry?.file_name();
        let mut filepath = PathBuf::from(posts_dir);
        filepath.push(&name);

        let mut out_name = name.to_owned();
        out_name.push(".html");

        info!("Rendering {:?} to {:?}", name, out_name);

        let source = fs::read_to_string(filepath)?;

        let (header, markdown) = split_md_and_header(&source)?;

        let meta = PostMeta {
            source_file: name.into_string().unwrap(),
            rendered_to: out_name.into_string().unwrap(),
            header,
        };

        let markdown_html = render_markdown(markdown);

        let result_html = render_markdown_into_template(markdown_html)?;

        write_output(&out_dir, &meta.rendered_to, result_html)?;

        post_metadata.push(meta);
    }

    Ok(post_metadata)
}

pub fn write_output(
    out_dir: impl AsRef<Path>,
    filename: impl AsRef<Path>,
    content: String,
) -> Result<(), Box<dyn Error>> {
    let out_dir = out_dir.as_ref();
    let filename = filename.as_ref();

    if let Err(e) = fs::read_dir(out_dir) {
        match e.kind() {
            io::ErrorKind::NotFound => {
                info!("Creating output directory {}", out_dir.display());
                fs::create_dir(out_dir)?;
            }
            _ => {
                panic!(
                    "Failed to access output directory {}: {}",
                    out_dir.display(),
                    e
                );
            }
        }
    };

    let mut filepath = PathBuf::from(out_dir);
    filepath.push(filename);
    let mut file = File::create(filepath)?;

    write!(file, "{}", content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::split_md_and_header;
    use std::str::FromStr;
    use toml::value::Datetime;

    #[test]
    fn test_split_md_and_header_should_read_meta() {
        let input = r#"---
title = "mytitle"
date = 2022-02-01
---
# heading"#
            .to_string();

        let (header, content) = split_md_and_header(&input).unwrap();

        assert!(header.is_some());

        let header = header.unwrap();

        assert_eq!(header.title.unwrap(), "mytitle".to_string());
        assert_eq!(
            header.date.unwrap(),
            Datetime::from_str("2022-02-01").unwrap()
        );

        assert_eq!(content, "# heading")
    }

    #[test]
    fn test_split_md_and_header_should_handle_no_meta() {
        let input = r"# heading".to_string();

        println!("{}", input);

        let (header, content) = split_md_and_header(&input).unwrap();

        assert!(header.is_none());

        assert_eq!(content, "# heading")
    }
}
