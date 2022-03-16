use std::collections::HashMap;

use log::{error, info};
use tera::{Context, Tera, Value};

use crate::{Post, PostHeader, TocHeading};

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

pub mod functions {
    /// tera function name for [`crate::templating::TocBuilder`]
    pub const MAKE_TOC: &str = "make_toc";
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

/// Tera function that generates a table of contents from [`TocHeading`]s
struct TocBuilder {
    pub headings: Vec<TocHeading>,
}

impl tera::Function for TocBuilder {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let open_list = &args["open_list"].as_str().unwrap_or("");
        let close_list = &args["close_list"].as_str().unwrap_or("");
        let open_list_item = &args["open_list_item"].as_str().unwrap_or("");
        let close_list_item = &args["close_list_item"].as_str().unwrap_or("");

        let mut skip_first = args.get("skip_first").map_or(false, |value| {
            value.as_bool().expect("failed to pass skip_first to bool")
        });

        let mut html = String::new();

        let mut open = 0;
        for heading in &self.headings {
            if skip_first {
                skip_first = false;
                continue;
            }
            if heading.level > heading.prev_level.unwrap_or(0) {
                html.push_str(open_list);
                open += 1;
            }

            if let Some(prev) = heading.prev_level {
                if heading.level < prev {
                    html.push_str(close_list);
                    open -= 1;
                }
            }

            html.push_str(open_list_item);
            html.push_str(heading.text.as_str());

            html.push_str(close_list_item);
        }

        for _ in 0..open {
            html.push_str(close_list);
        }

        Ok(Value::String(html))
    }

    fn is_safe(&self) -> bool {
        true
    }
}

/// replaces [`TocBuilder`] outside of post templates to prevent post headings leaking into other templates
/// if [`functions::MAKE_TOC`] is called
pub fn error_make_toc_fn_unavailable(_args: &HashMap<String, Value>) -> tera::Result<Value> {
    let msg = format!(
        "Looks like you are using {} outside of the post template, this is not supported",
        functions::MAKE_TOC
    );
    log::error!("{}", msg);
    Err(tera::Error::msg(msg))
}

pub fn render_markdown_into_template(
    tera: &mut Tera,
    context: &mut Context,
    header: &Option<PostHeader>,
    markdown: &str,
    headings: &[TocHeading],
) -> Result<String, tera::Error> {
    context.insert(values::POST_CONTENT, &markdown);
    context.insert(values::HEADER, &header);

    let toc_builder = TocBuilder {
        headings: headings.to_vec(),
    };
    tera.register_function(functions::MAKE_TOC, toc_builder);

    let result = tera.render(templates::POST, context);
    tera.register_function(functions::MAKE_TOC, error_make_toc_fn_unavailable);

    result
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

#[cfg(test)]
mod tests {
    use super::TocBuilder;
    use crate::TocHeading;
    use std::collections::HashMap;
    use tera::{Function, Value};

    fn test_headings() -> Vec<TocHeading> {
        // 1 (h1)
        // -1.1 (h2)
        // -1.1.1 (h3)
        // -1.2 (h2)
        vec![
            TocHeading {
                level: 1,
                prev_level: None,
                text: "1".to_string(),
            },
            TocHeading {
                level: 2,
                prev_level: Some(1),
                text: "1.1".to_string(),
            },
            TocHeading {
                level: 3,
                prev_level: Some(2),
                text: "1.1.1".to_string(),
            },
            TocHeading {
                level: 2,
                prev_level: Some(3),
                text: "1.2".to_string(),
            },
        ]
    }

    fn default_args() -> HashMap<String, Value> {
        let mut args = HashMap::new();

        args.insert(
            "open_list".to_string(),
            tera::Value::String("<ul>".to_string()),
        );
        args.insert(
            "close_list".to_string(),
            tera::Value::String("</ul>".to_string()),
        );
        args.insert(
            "open_list_item".to_string(),
            tera::Value::String("<li>".to_string()),
        );
        args.insert(
            "close_list_item".to_string(),
            tera::Value::String("</li>".to_string()),
        );

        args
    }

    #[test]
    fn test_toc_builder() {
        let toc_builder = TocBuilder {
            headings: test_headings(),
        };

        let args = default_args();

        let html = toc_builder.call(&args).expect("failed to call toc builder");

        let html: String = tera::from_value(html).unwrap();

        let mut expected = r"
        <ul>
            <li>1</li>
            <ul>
            <li>1.1</li>
                <ul>
                    <li>1.1.1</li>
                </ul>
            <li>1.2</li>
            </ul>
        </ul>"
            .to_string();

        remove_whitespace(&mut expected);
        assert_eq!(html, expected);
    }

    #[test]
    fn test_toc_builder_skip_first() {
        let toc_builder = TocBuilder {
            headings: test_headings(),
        };

        let mut args = default_args();

        args.insert("skip_first".to_string(), Value::from(true));

        let html = toc_builder.call(&args).expect("failed to call toc builder");

        let html: String = tera::from_value(html).unwrap();

        // first heading an outer ul is skipped
        let mut expected = r"
            <ul>
            <li>1.1</li>
                <ul>
                    <li>1.1.1</li>
                </ul>
            <li>1.2</li>
            </ul>"
            .to_string();

        remove_whitespace(&mut expected);
        assert_eq!(html, expected);
    }

    fn remove_whitespace(s: &mut String) {
        s.retain(|c| !c.is_whitespace());
    }
}
