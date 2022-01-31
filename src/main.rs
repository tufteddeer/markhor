use std::error::Error;

use rust_templating::render_markdown;

fn main() -> Result<(), Box<dyn Error>> {
    let markdown = "# title \n *strong*";

    let html = render_markdown(markdown);
    println!("{}", html);
    Ok(())
}
