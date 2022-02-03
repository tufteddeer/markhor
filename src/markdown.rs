use log::info;
use pulldown_cmark::{html, Parser};

use std::error::Error;
use std::fs;
use std::ops::Add;
use std::path::{Path, PathBuf};
use tera::Tera;

use crate::templating::render_markdown_into_template;
use crate::{write_output, PostHeader, PostMeta};

pub const MARKDOWN_HEADER_DELIMITER: &str = "---\n";

/// Convert markdown to html
///
/// # Examples
/// ```
/// use rust_templating::markdown::convert_markdown;
///
/// let md = "# Heading";
/// let html = convert_markdown(md);
///
/// assert_eq!(html, "<h1>Heading</h1>\n")
/// ```
pub fn convert_markdown(markdown: &str) -> String {
    let parser = Parser::new(markdown);

    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);

    html_out
}

/// Parse an optional [PostHeader] located at the start of a markdown document
/// (beginning and ending with [MARKDOWN_HEADER_DELIMITER] )
///
/// # Examples
///
/// ```
/// use rust_templating::markdown::split_md_and_header;
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

    if header == content {
        Ok((None, content))
    } else {
        let header: PostHeader = toml::from_str(header)?;
        Ok((Some(header), content))
    }
}

/// Convert every file in `posts_dir` to html and renders it into the `post.html` template
///
/// Returns a `Vec` of [PostMeta] info about converted posts
pub fn convert_posts(
    tera: &Tera,
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

        let out_name = match Path::new(&name).file_stem() {
            Some(s) => s.to_os_string().to_string_lossy().to_string(),
            None => name.to_string_lossy().to_string(),
        };
        let out_name = out_name.add(".html");

        info!("Rendering {:?} to {:?}", name, out_name);

        let source = fs::read_to_string(filepath)?;

        let (header, markdown) = split_md_and_header(&source)?;

        let markdown_html = convert_markdown(markdown);

        let result_html = render_markdown_into_template(tera, &header, markdown_html)?;

        let meta = PostMeta {
            source_file: name.into_string().unwrap(),
            rendered_to: out_name,
            header,
        };

        write_output(&out_dir, &meta.rendered_to, result_html)?;

        post_metadata.push(meta);
    }

    Ok(post_metadata)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use toml::value::Datetime;

    use crate::markdown::split_md_and_header;

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

        let (header, content) = split_md_and_header(&input).unwrap();

        assert!(header.is_none());

        assert_eq!(content, "# heading")
    }
}
