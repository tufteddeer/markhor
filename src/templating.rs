use log::{error, info};
use tera::{Context, Tera};

use crate::PostHeader;

pub fn init_tera(template_dir: &str) -> Tera {
    info!("Creating Tera");
    let mut tera = match Tera::new(template_dir) {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to create tera: {}", e);
            std::process::exit(1);
        }
    };

    tera.autoescape_on(vec![]);

    tera
}
pub fn render_markdown_into_template(
    tera: &Tera,
    context: &mut Context,
    header: &Option<PostHeader>,
    markdown: &str,
) -> Result<String, tera::Error> {
    context.insert("markdown_content", &markdown);
    context.insert("header", &header);

    tera.render("post.html", context)
}

pub fn render_index(tera: &Tera, context: &mut Context) -> Result<String, tera::Error> {
    tera.render("index.html", context)
}
