use chrono::NaiveDate;
use log::info;

use serde::{Deserialize, Serialize};

use std::cmp::Ordering::{self, Equal, Greater, Less};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{
    error::Error,
    fs::{self, File},
    io,
};

pub mod markdown;
pub mod templating;

/// PostHeader represents metadata added at the start of a markdown post.
#[derive(Debug, Serialize, Deserialize)]
pub struct PostHeader {
    pub title: Option<String>,
    pub date: Option<String>,
}

///
/// # Examples
///
/// ```
/// use rust_templating::PostHeader;
/// use std::cmp::Ordering::{self, Equal, Less, Greater};
/// use rust_templating::compare_header_date;
///
/// let a = PostHeader {title: None, date: Some("1900-01-01".to_string())};
/// let b = PostHeader {title: None, date: Some("2022-01-01".to_string())};
/// let c = PostHeader {title: None, date: Some("3333-01-01".to_string())};
/// let d = PostHeader {title: None, date: Some("1900-01-01".to_string())};
/// let e = PostHeader {title: None, date: None};
///
/// assert_eq!(compare_header_date(&a, &b), Less);
/// assert_eq!(compare_header_date(&a, &c), Less);
/// assert_eq!(compare_header_date(&a, &e), Greater);
/// assert_eq!(compare_header_date(&b, &a), Greater);
/// assert_eq!(compare_header_date(&b, &c), Less);
/// assert_eq!(compare_header_date(&e, &e), Equal);
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
#[derive(Debug, Serialize)]
pub struct PostMeta {
    /// the markdown file used as content source
    pub source_file: String,
    /// name of the rendered html file
    pub rendered_to: String,
    /// an optional [PostHeader] contained within the source file
    pub header: Option<PostHeader>,
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
/// use rust_templating::compare_option;
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
