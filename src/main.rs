use rust_templating::{render_markdown, render_markdown_into_template, write_output};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let markdown_file = "markdown/test.md";

    let markdown_html = render_markdown(markdown_file)?;

    let result_html = render_markdown_into_template(markdown_html)?;

    write_output("out", "hello.html".to_string(), result_html)
}
