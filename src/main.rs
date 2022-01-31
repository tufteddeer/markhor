use std::error::Error;

use rust_templating::render_template;

fn main() -> Result<(), Box<dyn Error>> {
    let result = render_template()?;
    println!("{}", result);
    Ok(())
}
