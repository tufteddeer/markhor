use tera::{Context, Tera};

pub fn render_template() -> Result<String, tera::Error> {
    let tera = Tera::new("templates/**/*.html")?;

    let mut context = Context::new();

    context.insert("name", "world");

    let animals = vec!["cat", "dog", "horse"];
    context.insert("animals", &animals);

    tera.render("hello.html", &context)
}
