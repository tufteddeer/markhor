use log::info;
use pulldown_cmark::{html, Parser, Tag};

use crate::{Post, PostHeader, PostMeta, TocHeading};
use pulldown_cmark::Event::End;
use pulldown_cmark::Event::Start;
use pulldown_cmark::Event::Text;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::ops::Add;
use std::path::{Path, PathBuf};

pub const MARKDOWN_HEADER_DELIMITER: &str = "---\n";

pub struct ConvertedMarkdown {
    pub content: String,
    pub headings: Vec<TocHeading>,
    pub preview_text: String,
}

/// Convert markdown to html
/// Extracts the first paragraph as preview text.
/// # Examples
/// ```
/// use yanos::markdown::convert_markdown;
///
/// let md = r"# Heading
/// ### second";
/// let converted = convert_markdown(md);
/// let headings = converted.headings;
/// let html = converted.content;
///
/// assert_eq!(html, "<h1>Heading</h1>\n<h2>second</h2>\n");
/// assert_eq!(headings.len(), 2);
/// assert_eq!(headings[0].level, 1);
/// assert_eq!(headings[0].prev_level, None);
/// assert_eq!(headings[0].text, "Heading");
///
/// assert_eq!(headings[1].level, 2);
/// assert_eq!(headings[1].prev_level, Some(1));
/// assert_eq!(headings[1].text, "second");
/// ```
pub fn convert_markdown(markdown: &str) -> ConvertedMarkdown {
    let parser = Parser::new(markdown);

    let mut in_heading = false;

    let mut headings = Vec::<TocHeading>::new();
    // string to collect all the text inside a heading (if there are nested tags inside the h tags)
    // will keep the text, but loose the tags
    let mut current_heading = String::new();

    #[derive(PartialEq, Eq)]
    enum PreviewReadingState {
        Searching,
        Reading,
        Complete,
    }

    let mut preview_reading_state = PreviewReadingState::Searching;

    let mut first_paragraph = String::new();

    let iterator = parser.map(|event| {
        match &event {
            Start(Tag::Heading(_, _, _)) => {
                in_heading = true;
                current_heading = String::new();
            }
            End(Tag::Heading(level, _, _)) => {
                in_heading = false;
                if !current_heading.is_empty() {
                    let s = level.to_string();
                    let lvl_num = s
                        .strip_prefix('h')
                        .expect("failed to strip h from heading tag");
                    let lvl_num = lvl_num
                        .parse::<u8>()
                        .expect("failed to parse heading level to int");

                    let prev_level = headings.last().map(|prev| prev.level);

                    let toc_entry = TocHeading {
                        level: lvl_num,
                        prev_level,
                        text: current_heading.clone(),
                    };
                    headings.push(toc_entry);
                }
            }
            Start(Tag::Paragraph) => {
                if preview_reading_state == PreviewReadingState::Searching {
                    preview_reading_state = PreviewReadingState::Reading;
                }
            }
            End(Tag::Paragraph) => {
                if preview_reading_state == PreviewReadingState::Reading {
                    preview_reading_state = PreviewReadingState::Complete;
                }
            }
            Text(text) => {
                if in_heading {
                    current_heading.push_str(text);
                }
                if preview_reading_state == PreviewReadingState::Reading {
                    first_paragraph.push_str(text);
                }
            }
            _ => {}
        }

        event
    });

    let mut html_out = String::new();
    html::push_html(&mut html_out, iterator);

    ConvertedMarkdown {
        content: html_out,
        headings,
        preview_text: first_paragraph,
    }
}

/// Parse an optional [PostHeader] located at the start of a markdown document
/// (beginning and ending with [MARKDOWN_HEADER_DELIMITER] )
///
/// # Examples
///
/// ```
/// use yanos::markdown::split_md_and_header;
/// use std::str::FromStr;
/// use toml::value::Datetime;
/// let input = r#"---
/// title = "mytitle"
/// category = "mycategory"
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
/// assert_eq!(header.category.unwrap(), "mycategory".to_string());
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

/// Convert every file in `posts_dir` to html, generates meta info and the html representation
///
/// Returns a [HashMap] of [Post]s, keyed by category
pub fn convert_posts(
    posts_dir: impl AsRef<Path>,
) -> Result<BTreeMap<Option<String>, Vec<Post>>, Box<dyn Error>> {
    let posts_dir = posts_dir.as_ref();

    let mut posts = BTreeMap::<Option<String>, Vec<Post>>::new();

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

        info!("Setting out_name for {:?} to {:?}", name, out_name);

        let source = fs::read_to_string(&filepath)?;

        let (header, markdown) = split_md_and_header(&source)?;

        let mut category = None;
        let mut out_path = if let Some(h) = &header {
            if let Some(cat) = &h.category {
                info!("Post {} has category {}", filepath.display(), cat);
                category = Some(cat.to_string());
                PathBuf::from(cat)
            } else {
                PathBuf::new()
            }
        } else {
            PathBuf::new()
        };

        out_path.push(out_name);

        let converted_md = convert_markdown(markdown);

        let meta = PostMeta {
            source_file: name.into_string().unwrap(),
            rendered_to: out_path.to_string_lossy().to_string(),
            header,
            preview_text: converted_md.preview_text,
        };

        let post = Post {
            meta,
            content: converted_md.content,
            headings: converted_md.headings,
        };

        match posts.get_mut(&category) {
            Some(postvec) => {
                postvec.push(post);
            }
            None => {
                posts.insert(category, vec![post]);
            }
        }
    }

    Ok(posts)
}

#[cfg(test)]
mod tests {

    use crate::markdown::split_md_and_header;

    #[test]
    fn test_split_md_and_header_should_read_meta() {
        let input = r#"---
title = "mytitle"
date = "2022-02-01"
---
# heading"#
            .to_string();

        let (header, content) = split_md_and_header(&input).unwrap();

        assert!(header.is_some());

        let header = header.unwrap();

        assert_eq!(header.title.unwrap(), "mytitle".to_string());
        assert_eq!(header.date.unwrap(), "2022-02-01".to_string());

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
