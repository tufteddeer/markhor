use std::error::Error;

use tera::{Context, Tera};

fn main() -> Result<(), Box<dyn Error>> {
    let tera = Tera::new("templates/**/*.html").expect("failed to parse template");

    let mut context = Context::new();

    context.insert("name", "world");

    let animals = vec!["cat", "dog", "horse"];
    context.insert("animals", &animals);
    let result = tera.render("hello.html", &context)?;

    println!("{}", result);
    Ok(())
}
