use std::{error::Error, fs};

use pulldown_cmark::{html, Parser};
use tera::{Context, Tera};

pub fn render_template() -> Result<String, tera::Error> {
    let tera = Tera::new("templates/**/*.html")?;

    let mut context = Context::new();

    context.insert("name", "world");

    let animals = vec!["cat", "dog", "horse"];
    context.insert("animals", &animals);

    tera.render("hello.html", &context)
}

pub fn render_markdown(filename: &str) -> Result<String, Box<dyn Error>> {
    let input = fs::read_to_string(filename)?;

    let parser = Parser::new(&input);

    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);

    Ok(html_out)
}
