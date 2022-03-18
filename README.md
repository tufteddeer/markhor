# Yanos (**y**et **ano**ther **s**tatic site generator)

[![Clippy check](https://github.com/tufteddeer/yanos/actions/workflows/clippy.yml/badge.svg)](https://github.com/tufteddeer/yanos/actions/workflows/clippy.yml)
[![Tests](https://github.com/tufteddeer/yanos/actions/workflows/test.yml/badge.svg)](https://github.com/tufteddeer/yanos/actions/workflows/test.yml)
[![Docker build](https://github.com/tufteddeer/yanos/actions/workflows/docker.yaml/badge.svg)](https://github.com/tufteddeer/yanos/actions/workflows/docker.yaml)

## Features

- Markdown files in `posts` are rendered into the `post.html` template (using [pulldown-cmark](https://crates.io/crates/pulldown-cmark))
- Posts can have an optional `title` and `date` attribute
- A list of posts is rendered in the `index.html` template (newest first)
- Templates use the [Tera](https://tera.netlify.app/) template engine
- Static content directory (`static`) is copied to `out/static`
- Serve generated site (`--serve`, only for development purposes)
- Watch files and regenerate on changes (`--watch`)
- Automatically extracted preview texts for posts (the first paragraph)

## Variables

| **Variable**      | **Template**          |
| ----------------- | --------------------- |
| posts_in_category | category              |
| post_categories   | post, category, index |
| category          | category              |

## Functions

| **Function** | **Template** |
| ------------ | ------------ |
| make_toc     | post         |


### make_toc
`make_toc` can be used to create a table of contents inside the _post_ template. Headings are automatically extracted from markdown during conversion.

Begin and end html code for items and lists can be configured via function arguments.

```markdown

# Heading 1

## Heading 1.1
### Heading 1.1.2

## Heading 1.2

```

A Post containing the above markdown headings that envokes `make_toc` like this:
```html
{{ make_toc(
    open_list = "<ul>",
    close_list = "</ul>",
    open_list_item = "<li>",
    close_list_item = "</li>",
 ) }}
```
will produce the following html:

```html

<ul>
    <li>Heading 1</li>
    <ul>
        <li>Heading 1.1</li>
    <ul>
        <li>Heading 1.1.2</li>
    </ul>
        <li>Heading 1.2</li>
    </ul>
</ul>
```

The first heading can be excluded from the table of contents using the optional `skip_first=true` argument.
