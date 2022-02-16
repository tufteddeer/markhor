use std::{
    fs, io,
    ops::Sub,
    path::{Path, PathBuf},
    time::Instant,
};

use clap::Parser;
use fs_extra::{copy_items, dir};
use log::info;
use simple_logger::SimpleLogger;
use tera::Context;
#[cfg(feature = "serve")]
use yanos::serve::serve_files;
use yanos::{
    compare_header_date, compare_option,
    markdown::convert_posts,
    templating::{self, render_category_page, render_index, render_markdown_into_template, values},
    write_output, PostMeta,
};

const POSTS_DIR: &str = "posts";
const OUT_DIR: &str = "out";
const STATIC_DIR: &str = "static";
const TEMPLATES_GLOB: &str = "templates/**/*";

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// Serve generated files
    #[clap(long)]
    serve: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    SimpleLogger::new()
        .with_module_level("globset", log::LevelFilter::Error)
        .init()?;

    let posts_dir = Path::new(POSTS_DIR);
    let output_dir = Path::new(OUT_DIR);

    let start_time = Instant::now();

    let tera = templating::init_tera(TEMPLATES_GLOB);

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

            let dir = PathBuf::from(output_dir);

            write_output(dir, &meta.rendered_to, result_html)?;
        }

        context.remove(values::POST_CONTENT);
        context.remove(values::HEADER);

        if let Some(cat) = category {
            let category_page_html = render_category_page(&tera, &mut context, cat, posts)?;

            let category_out_file = format!("{cat}.html");
            write_output(OUT_DIR, category_out_file, category_page_html)?;
        }
    }

    let index_html = render_index(&tera, &mut context)?;

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

    if args.serve {
        #[cfg(feature = "serve")]
        serve_files("127.0.0.1:8080", output_dir)?;
        #[cfg(not(feature = "serve"))]
        log::error!("Feature 'serve' was not enabled during compilation. Server not available");
    }

    Ok(())
}
