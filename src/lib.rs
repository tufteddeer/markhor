use pulldown_cmark::{html, Parser};
use std::io::Write;
use std::path::PathBuf;
use std::{
    error::Error,
    fs::{self, File},
    io,
};
use tera::{Context, Tera};

pub fn render_markdown_into_template(markdown: String) -> Result<String, tera::Error> {
    let mut tera = Tera::new("templates/**/*.html")?;

    let mut context = Context::new();

    context.insert("markdown_content", &markdown);

    let animals = vec!["cat", "dog", "horse"];
    context.insert("animals", &animals);

    tera.autoescape_on(vec![]);
    tera.render("hello.html", &context)
}

pub fn render_markdown(filename: &str) -> Result<String, Box<dyn Error>> {
    let input = fs::read_to_string(filename)?;

    let parser = Parser::new(&input);

    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);

    Ok(html_out)
}

pub fn write_output(
    out_dir: &str,
    filename: String,
    content: String,
) -> Result<(), Box<dyn Error>> {
    if let Err(e) = fs::read_dir(out_dir) {
        match e.kind() {
            io::ErrorKind::NotFound => {
                println!("dir does not exist");
                fs::create_dir(out_dir)?;
            }
            _ => {
                panic!("Failed to access output directory {}: {}", out_dir, e);
            }
        }
    };

    let mut filepath = PathBuf::from(out_dir);
    filepath.push(filename);
    let mut file = File::create(filepath)?;

    write!(file, "{}", content)?;

    Ok(())
}
