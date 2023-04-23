---
title: "How to make a blog in Rust"
description: "Yes, I'm talking about this very blog. A journey about how scaling down and settling with compromises can turn out beautiful."
date: "2023-04-22"
---

# {{page.metadata.title}}

The brains of this blog is a static site generator that I wrote. 

You can find the complete code on my [GitHub](https://github.com/BRA1L0R), this post will just outline the main challenges and personal considerations I made along the way.

## The original idea

<!-- When I come out with new ideas they are always utopian conceptions of what I'll actually do.  -->

I originally intended this blog to have dynamic server-side rendered features and run on WebAssembly with leptos. 

Yes, all of this just for a blog.

Deep inside I knew it was going to be just another half-finished project, as all non-rationalized ideas of mine end up being. Thrown in a bin because trying to bite off more than I could chew (or that I should have ever had for a blog).

<!-- There's a difference between setting yourself unachiavable goals and setting yourself a challenge. The line is drawn by the ratio between **time** and **reward**. -->

<!-- It's a pity that many good ideas of mine get eaten by frustration and  -->

## Scaling down

Browsing through other blog sources written by fellow rustaceans I noticed a recurring pattern: **static site generators**.

Even the [Rust blog](https://github.com/rust-lang/blog.rust-lang.org) is structured as such. This is where I drew my inspiration for serving the content, although I used [tokio](docs.rs/tokio) and [rocket](docs.rs/rocket/0.5.0-rc.3) whilst they [do not](https://github.com/rust-lang/blog.rust-lang.org/blob/master/serve/src/main.rs).

But how could I ever scale down to such a rudimentary technology? The world isn't built upon static site generators.

But <mark>I'm not the world</mark>.

It might stupid at first, but once you realize how the weight of the "world" sliding down your shoulders as you settle down to a mediocre compromise comforts you, only then you can fathom the beauty of mediocrity[^1].

As you probably already figured out, this isn't just about blogs. It is a lesson about rationalizing ideas into achievable goals that fit the desired outcome.

## Idea: Handlebars + Markdown!

Ok, let's get to the **technical stuff**.

Even though I settled for a static site generator I still did not want to let my original **Markdown** idea go. And in fact I didn't.

I still had to figure out how and where to index pages, but since I was already using Handlebars to template HTML I decided to use it to hydrate Markdown with it too:

{{{{raw}}}}
```markdown
# {{page.metadata.title}}

List of blog pages:
{{#each pages}}
- {{metadata.title}}: {{metadata.description}}
{{/each}}
```
{{{{/raw}}}}

If you aren't familiar with Handlebar I suggest [reading the official guide](https://handlebarsjs.com/guide/expressions.html#basic-usage).

## The code

Generating the final `dist/` folder containing html pages can be subdivided into these steps:
1. Indexing:
    1. **Indexing** through the `pages/` directory with the help of [walkdir](docs.rs/walkdir) ![](/static/crate-small.svg)
    2. Separating the [**frontmatter**](https://daily-dev-tips.com/posts/what-exactly-is-frontmatter/) (yaml metadata) from the content
1. Hydrating and rendering:
    1. **Sorting** pages by **date** into a `Vec<&Page>` of references
    2. **Hydrating** the Markdown with the help of [handlebars](docs.rs/handlebars) ![](/static/crate-small.svg)
    3. **Compiling** Markdown into HTML with [markdown](docs.rs/markdown/1.0.0-alpha.8) ![](/static/crate-small.svg)
    4. **Hydrating** the HTML template with the compiled markdown

Walking through a directory is easy thanks to `walkdir`. I just had to filter out folders:

```rs
let directory = WalkDir::new(dir)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|entry| entry.path().is_file());

for file in directory {
    ...
}
```

I read all pages into Strings, strip the base from their path and put them in a `Vec<Page>`.

I also separate and decode the frontmatter here. I decode it using [serde_yaml](docs.rs/serde_yaml) ![](/static/crate-small.svg) into a `Metadata` struct defined like this:

```rs
#[derive(Deserialize, Serialize)]
struct Metadata {
    title: String,
    description: String,
    date: Date,
    #[serde(default)]
    tags: Vec<String>,

    #[serde(default)]
    hidden: bool,
}
```

The `hidden` attribute can be used by Handlebars code to determine if a given page should be listed (e.g. the Index page isn't).

serde_yaml supports borrowed deserialization, allowing for less allocations. But I couldn't make use of it as I need all `Page`'s to be `'static` and allocated inside a `Vec<Page>` for indexing and sorting.

Besides, the generator meant to be run **one-shot**, and as long as I can rationally justify these expenses it's fine.

I explicitly opted for a **strongly-typed** `Date` type from the [time](docs.rs/time) ![](/static/crate-small.svg) crate and enabled the `serde-human-readable` feature so that every date is formatted **the same way**.

Since `Page` weighs ~150 bytes, sorting that `Vec` directly is unecessarily wasteful. So I create a `Vec<&Page>` and sort them by date:

```rust
#[derive(Deserialize, Serialize, Debug)]
struct Page {
    figurative_path: PathBuf,
    absolute_path: PathBuf,

    metadata: Metadata,
    raw: String,
}

let pages: Vec<Page>; // defined somewhere

let mut sorted: Vec<&Page> = pages.iter().collect();
sorted.sort_by_key(|&a| std::cmp::Reverse(a.metadata.date));
```

`Vec<Page>` is perfectly sized iterator so `Vec<&Page>` gets preallocated with a matching capacity, reducing wasteful allocations. Also, I make use of [`std::cmp::Reverse`](https://doc.rust-lang.org/std/cmp/struct.Reverse.html) which inverts the output of the `PartialOrd` trait.

Templating and compiling is pretty straight forward. The HTML template looks something like this:

{{{{raw}}}}
```html
<html> 
    <head>
        <title>{{title}}</title>
        <style>{{{style}}}</style>
    </head>
    <body>{{{content}}}</body>
</html>
```
{{{{/raw}}}}

Notice the triple brackets on `content` and `style`, allowing unescaped HTML and CSS to be inserted.

Direct embedding of CSS allows for (mostly) self-contained pages, which I thought fit the minimalist conception of this project.

And in-code:
```rust
let mut templating = Handlebars::new();
templating
    .register_template_string(TEMPLATE_NAME, TEMPLATE)
    .unwrap();

for page in pages { 
    let hydrated = templating.render_template(&page.raw, &data)?;
    // never unwraps without MDX enabled
    let compiled = markdown::to_html_with_options(&hydrated, &options).unwrap(); 

    let mut file = File::create(output_path)?;
    templating.render_to_write(TEMPLATE_NAME, &data, &mut file)?;
}
```

## Deploying

Since most of the times I commit a new post I won't change any SSG code I made sure to make full use of Docker's caching capabilities.

You can find the full Dockerfile in the repository, let's break down the main parts.

```dockerfile
FROM rust:latest as serve

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev

COPY serve/ /serve/
WORKDIR /serve
RUN cargo build --release --target x86_64-unknown-linux-musl
```

I create a "serve" stage from [`rust:latest`](https://hub.docker.com/_/rust) and install the MUSL toolchain. I do this because I want the final image to be based off `alpine`, as the rust one is pretty heavy.

```dockerfile
FROM rust:latest as builder

WORKDIR /build

# cache build if nothing changed but pages
COPY Cargo.toml Cargo.toml
COPY Cargo.toml Cargo.toml
COPY src/ src/
RUN cargo build --release

COPY pages/ pages/
RUN cargo run --release
```

I seperate building and running the SSG so that if I do not change any of the files before `RUN cargo build --release` it will cache the binary output and always re-use the same one. I then build the pages that will output in the `dist/` folder.

```dockerfile
FROM alpine

# copy binaries, dist/ and static/

ENV SERVE_DIR /app
EXPOSE 8000

CMD ["./serve"]
```

Last but not least, I build the final image from [`alpine`](https://hub.docker.com/_/alpine).

The output image is very lightweight (~18MB in size) and ready to serve.

## Conclusions

I'm very happy with the versatility and simplicity the union of Markdown and Handlebars offers. 

I skipped how I implemented syntax highlighting because I'd rather server-side render it but am using `highlight.js` and I'll probably change it in the future.

---

[^1]: _mĕdĭŏcris_ is a Latin word for "middle, ordinary, common". 
It isn't to be thought as a derogatory term, although It can mean it in many contexts.