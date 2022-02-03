use log::info;

use serde::{Deserialize, Serialize};

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
