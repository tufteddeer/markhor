use chrono::NaiveDate;
use fs_extra::{copy_items, dir};
use log::info;

use markdown::convert_posts;
use serde::{Deserialize, Serialize};
use templating::{render_index, values};
use tera::Context;

use std::cmp::Ordering::{self, Equal, Greater, Less};
use std::io::Write;
use std::ops::Sub;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::{
    error::Error,
    fs::{self, File},
    io,
};

use crate::templating::{render_category_page, render_markdown_into_template};

pub mod markdown;
#[cfg(feature = "serve")]
pub mod serve;
pub mod templating;
#[cfg(feature = "watch")]
pub mod watch;

/// PostHeader represents metadata added at the start of a markdown post.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct PostHeader {
    pub title: Option<String>,
    pub date: Option<String>,
    pub category: Option<String>,
}

#[derive(Serialize)]
pub struct Post {
    pub meta: PostMeta,
    pub content: String,
}

///
/// # Examples
///
/// ```
/// use yanos::PostHeader;
/// use std::cmp::Ordering::{self, Equal, Less, Greater};
/// use yanos::compare_header_date;
///
/// let a = PostHeader {title: None, date: Some("1900-01-01".to_string()), category: None};
/// let b = PostHeader {title: None, date: Some("2022-01-01".to_string()), category: None};
/// let c = PostHeader {title: None, date: Some("3333-01-01".to_string()), category: None};
/// let d = PostHeader {title: None, date: Some("1900-01-01".to_string()), category: None};
/// let e = PostHeader {title: None, date: None, category: None};
///
/// assert_eq!(compare_header_date(&a, &b), Less);
/// assert_eq!(compare_header_date(&a, &c), Less);
/// assert_eq!(compare_header_date(&a, &e), Greater);
/// assert_eq!(compare_header_date(&b, &a), Greater);
/// assert_eq!(compare_header_date(&b, &c), Less);
/// assert_eq!(compare_header_date(&e, &e), Equal);
/// 
/// #[cfg(feature = "serve")]
/// assert!(false);
/// ```
pub fn compare_header_date(a: &PostHeader, b: &PostHeader) -> Ordering {
    let da = a.date.as_ref();
    let db = b.date.as_ref();

    compare_option(&da, &db, |a, b| {
        let date_a = NaiveDate::parse_from_str(a, "%Y-%m-%d").expect("failed to parse date");
        let date_b = NaiveDate::parse_from_str(b, "%Y-%m-%d").expect("failed to parse date");

        date_a.cmp(&date_b)
    })
}

/// PostMeta contains post metadata originated from the build process and the optional [PostHeader]
#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
pub struct PostMeta {
    /// the markdown file used as content source
    pub source_file: String,
    /// name of the rendered html file
    pub rendered_to: String,
    /// an optional [PostHeader] contained within the source file
    pub header: Option<PostHeader>,
}

pub fn generate_site<P>(
    templates_glob: &str,
    posts_dir: P,
    output_dir: P,
) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path> + Copy,
{
    let start_time = Instant::now();

    let tera = templating::init_tera(templates_glob);

    let posts_by_cat = convert_posts(posts_dir)?;

    let mut sorted_meta: Vec<&PostMeta> = posts_by_cat
        .iter()
        .flat_map(|(_, postvec)| postvec.iter().map(|post| &post.meta))
        .collect();
    sorted_meta.sort_unstable_by(|a, b| {
        compare_option(&b.header, &a.header, |meta_a, meta_b| {
            compare_header_date(meta_a, meta_b)
        })
    });

    let categories: Vec<&Option<String>> = posts_by_cat.keys().into_iter().collect();

    let mut context = Context::new();
    context.insert(values::POSTS_META, &sorted_meta);
    context.insert(values::POST_CATEGORIES, &categories);

    for (category, posts) in &posts_by_cat {
        info!("Rendering category: {:?}", category);

        for post in posts {
            let meta = &post.meta;
            let content = &post.content;
            let result_html =
                render_markdown_into_template(&tera, &mut context, &meta.header, content)?;

            write_output(&output_dir, &meta.rendered_to, result_html)?;
        }

        context.remove(values::POST_CONTENT);
        context.remove(values::HEADER);

        if let Some(cat) = category {
            let category_page_html = render_category_page(&tera, &mut context, cat, posts)?;

            let category_out_file = format!("{cat}.html");
            write_output(output_dir, category_out_file, category_page_html)?;
        }
    }

    let index_html = render_index(&tera, &mut context)?;

    write_output(output_dir, "index.html", index_html)?;

    let elapsed_time = Instant::now().sub(start_time);
    log::info!("Took {}ms", &elapsed_time.as_millis());
    Ok(())
}

pub fn write_output(
    out_dir: impl AsRef<Path>,
    filename: impl AsRef<Path>,
    content: String,
) -> Result<(), Box<dyn Error>> {
    let out_dir = out_dir.as_ref();
    let filename = filename.as_ref();

    let mut filepath = PathBuf::from(out_dir);
    filepath.push(filename);

    // file target directory is the general out_dir, possibly followed by
    // an optional subfolder includes in the filename
    let mut out_file_dir = filepath.clone();
    out_file_dir.pop();
    let out_file_dir = out_file_dir.as_path();

    if let Err(e) = fs::read_dir(out_file_dir) {
        match e.kind() {
            io::ErrorKind::NotFound => {
                info!("Creating output directory {}", out_file_dir.display());
                fs::create_dir_all(out_file_dir)?;
            }
            _ => {
                panic!(
                    "Failed to access output directory {}: {}",
                    out_file_dir.display(),
                    e
                );
            }
        }
    };

    let mut file = File::create(filepath)?;

    write!(file, "{}", content)?;

    Ok(())
}

pub fn copy_static_files<P>(static_dir: P, out_dir: P) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path> + Copy,
{
    if let Err(e) = fs::read_dir(static_dir) {
        match e.kind() {
            io::ErrorKind::NotFound => {
                info!("No static directory found, skipping");
            }
            _ => {
                panic!(
                    "Failed to access static directory {}: {}",
                    &static_dir.as_ref().display(),
                    e
                );
            }
        }
    } else {
        info!("Copying static assets");

        let mut options = dir::CopyOptions::new();
        options.overwrite = true;

        let from = vec![static_dir];
        copy_items(&from, out_dir, &options)?;
    }

    Ok(())
}

/// Compare two [Option]s with values that don't implement `Eq`, `Ord` etc.
/// ## Rules
/// - None == None
/// - None < Some
/// - If both are `Some`, the content is passed to `some_cmp` for comparison
///
/// # Examples
/// ```
/// use core::cmp::Ordering::Less;
/// use std::cmp::Ordering::Greater;
/// use std::cmp::Ordering::Equal;
/// use yanos::compare_option;
///
/// let some_high = Some(100);
/// let some_low = Some(1);
///
/// assert_eq!(Greater, compare_option(&some_high, &None, |a, b| {
///     a.cmp(b)
/// }));
///
/// assert_eq!(Greater, compare_option(&some_high, &some_low, |a, b| {
///     a.cmp(b)
/// }));
/// assert_eq!(Equal, compare_option(&some_high, &Some(100), |a, b| {
///     a.cmp(b)
/// }));
///
/// assert_eq!(Greater, compare_option(&some_low, &None, |a, b| {
///     a.cmp(b)
/// }));
/// ```
pub fn compare_option<T, F>(a: &Option<T>, b: &Option<T>, some_comp: F) -> Ordering
where
    F: Fn(&T, &T) -> Ordering,
{
    if a.is_none() && b.is_none() {
        return Equal;
    };
    if a.is_none() {
        return Less;
    };
    if b.is_none() {
        return Greater;
    };

    some_comp(a.as_ref().unwrap(), b.as_ref().unwrap())
}
