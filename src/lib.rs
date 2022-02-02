use log::info;

use serde::{Deserialize, Serialize};

use std::io::Write;
use std::path::{Path, PathBuf};
use std::{
    error::Error,
    fs::{self, File},
    io,
};
use toml::value::Datetime;

pub mod markdown;
pub mod templating;

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
