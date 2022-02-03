# Yanos (**y**et **ano**ther  **s**tatic site generator)

## Features

- Markdown files in `posts` are rendered into the `post.html` template (using [pulldown-cmark](https://crates.io/crates/pulldown-cmark))
- Posts can have an optional `title` and `date` attribute
- A list of posts is rendered in the `index.html` template (newest first)
- Templates use the [Tera](https://tera.netlify.app/) template engine
- Static content directory (`static`) is copied to `out/static`