# Markhor

[![Clippy check](https://github.com/tufteddeer/markhor/actions/workflows/clippy.yml/badge.svg)](https://github.com/tufteddeer/markhor/actions/workflows/clippy.yml)
[![Tests](https://github.com/tufteddeer/markhor/actions/workflows/test.yml/badge.svg)](https://github.com/tufteddeer/markhor/actions/workflows/test.yml)
[![Docker build](https://github.com/tufteddeer/markhor/actions/workflows/docker.yaml/badge.svg)](https://github.com/tufteddeer/markhor/actions/workflows/docker.yaml)

## Features

- Markdown files in `posts` are rendered into the `post.html` template (using [pulldown-cmark](https://crates.io/crates/pulldown-cmark))
- Posts can have an optional `title` and `date` attribute
- A list of posts is rendered in the `index.html` template (newest first)
- Templates use the [Tera](https://tera.netlify.app/) template engine
- Static content directory (`static`) is copied to `out/static`
- Serve generated site (`--serve`, only for development purposes)
- Watch files and regenerate on changes (`--watch`)
- Automatically extracted preview texts for posts (the first paragraph)
- draft support

## Usage

The source directory is expected to look like this:

[Example](https://github.com/tufteddeer/tufteddeer.github.io) (my github.io page)

```
.
├── posts
│   ├── first-post.md
│   ├── some-other-post.md
├── static
│   ├── image.png
│   └── style.css
└── templates
    ├── base.html
    ├── category.html
    ├── index.html
    └── post.html
```

Generating the site using

```bash
markhor 
```

in the directory, will place generated html in the `out` directory.

```
out
├── first-post.html
├── second-post.html
├── index.html
└── static
    ├── image.png
    └── style.css
```

Posts that have a category assigned in their header will be put in a subdirectory of `out`, named after the category. A _category-name_.html file will be generated in `out` using the `category` template.

### Posts

A post is a markdown file located in `posts/`.
The markdown flavor is [CommonMark](https://commonmark.org/)

A header can be added at the top of the file and is available as `header` in the template (see [variables](#template-variables)).

```
---
title = "Hello world"
date = "2022-02-01"
---
# My first Post

your text goes here
```
The first paragraph will automatically be available in the post metadata as `preview_text`.

A post can be marked as _draft_ by setting `draft = true` in the header. Drafts can be included in the build using the `--drafts` flag.

### Preview

Building the site using

```
markhor --serve --watch
```

will start a webserver available at http://localhost:8080, serving your site. This is for development purposes only and should not be used to host your website on the internet.

The `--watch` flag will automatically rebuild your site when files in `posts`, `static` or `templates` change.

Both flags work independent from another.

### Docker

A docker image is built automatically from new tags and available as `ghcr.io/tufteddeer/markhor:latest`.

### Building a site using GitHub actions

A GitHub action is available at [markhor-action](https://github.com/tufteddeer/markhor-action)

## Templates

* `post.html` will be used for markdown content in `posts/`
* `category.html` is the basis for category landing pages
* `index.html` will be used to generate the sites `index.html`

See the [tera docs](https://tera.netlify.app/docs/) for documentation concerning the general usage of templates.

## Template variables

| **Variable**      | **Template**          | **Value**
| ----------------- | --------------------- |---------------------
| posts_in_category | category              | List of all posts in the current category
| category          | category              | The current category
| post_categories   | post, category, index | List of all categories
| markdown_content  | post                  | Post content from markdown file, as HTML
| posts_meta        | post, category, index | Metadata about every post, sorted newest first
| header            | post                  | Post header

## Template functions

| **Function** | **Template** |
| ------------ | ------------ |
| [make_toc](#make_toc)     | post         |


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
