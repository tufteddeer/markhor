use std::error::Error;

use rust_templating::render_markdown;

fn main() -> Result<(), Box<dyn Error>> {
    let markdown = "markdown/test.md";

    let html = render_markdown(markdown)?;
    println!("{}", html);
    Ok(())
}
