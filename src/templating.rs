use log::{error, info};
use tera::{Context, Tera};

use crate::{Post, PostHeader};

pub mod templates {
    pub const INDEX: &str = "index.html";
    pub const POST: &str = "post.html";
    pub const CATEGORY: &str = "category.html";
}

pub mod values {
    /// Content of a post, e.g. from a markdown file
    pub const POST_CONTENT: &str = "markdown_content";
    /// [`crate::PostHeader`]
    pub const HEADER: &str = "header";
    /// current category (for category overview pages)
    pub const CATEGORY: &str = "category";
    /// all [crate::Post]s of the currently rendered category
    pub const POSTS_IN_CATEGORY: &str = "posts_in_category";
    /// all categories
    pub const POST_CATEGORIES: &str = "post_categories";
    /// metadata for all posts
    pub const POSTS_META: &str = "posts_meta";
}

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
    context.insert(values::POST_CONTENT, &markdown);
    context.insert(values::HEADER, &header);

    tera.render(templates::POST, context)
}

pub fn render_index(tera: &Tera, context: &mut Context) -> Result<String, tera::Error> {
    tera.render(templates::INDEX, context)
}

pub fn render_category_page(
    tera: &Tera,
    context: &mut Context,
    category: &str,
    posts: &[Post],
) -> Result<String, tera::Error> {
    context.insert(values::CATEGORY, category);
    context.insert(values::POSTS_IN_CATEGORY, &posts);

    let category_page = tera.render(templates::CATEGORY, context)?;

    context.remove(values::CATEGORY);
    context.remove(values::POSTS_IN_CATEGORY);

    Ok(category_page)
}
