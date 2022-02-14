use std::{fs, io, ops::Sub, path::Path, time::Instant};

use fs_extra::{copy_items, dir};
use log::info;
use simple_logger::SimpleLogger;
use tera::Context;
use yanos::{
    compare_header_date, compare_option,
    markdown::convert_posts,
    templating::{self, render_index, render_markdown_into_template},
    write_output, PostMeta,
};

const POSTS_DIR: &str = "posts";
const OUT_DIR: &str = "out";
const STATIC_DIR: &str = "static";
const TEMPLATES_GLOB: &str = "templates/**/*";
const NUM_LATEST_POSTS: usize = 10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_module_level("globset", log::LevelFilter::Error)
        .init()?;

    let posts_dir = Path::new(POSTS_DIR);
    let output_dir = Path::new(OUT_DIR);

    let start_time = Instant::now();

    let tera = templating::init_tera(TEMPLATES_GLOB);

    let (mut meta, posts) = convert_posts(posts_dir)?;

    let mut latest = meta.clone();
    latest.sort_unstable_by(|a, b| {
        compare_option(&b.header, &a.header, |meta_a, meta_b| {
            compare_header_date(meta_a, meta_b)
        })
    });
    let latest: Vec<&PostMeta> = latest.iter().take(NUM_LATEST_POSTS).collect();

    let mut context = Context::new();
    context.insert("latest_posts", &latest);

    for i in 0..posts.len() {
        let m = &meta[i];
        let p = &posts[i];

        let result_html = render_markdown_into_template(&tera, &mut context, &m.header, p)?;

        write_output(output_dir, &m.rendered_to, result_html)?;
    }

    meta.sort_unstable_by(|a, b| {
        compare_option(&b.header, &a.header, |meta_a, meta_b| {
            compare_header_date(meta_a, meta_b)
        })
    });

    let index_html = render_index(&tera, &meta)?;

    write_output(output_dir, "index.html", index_html)?;

    let elapsed_time = Instant::now().sub(start_time);
    log::info!("Took {}ms", &elapsed_time.as_millis());

    if let Err(e) = fs::read_dir(STATIC_DIR) {
        match e.kind() {
            io::ErrorKind::NotFound => {
                info!("No static directory found, skipping");
            }
            _ => {
                panic!("Failed to access static directory {}: {}", STATIC_DIR, e);
            }
        }
    } else {
        info!("Copying static assets");

        let mut options = dir::CopyOptions::new();
        options.overwrite = true;

        let from = vec![STATIC_DIR];
        copy_items(&from, output_dir, &options)?;
    }

    Ok(())
}
