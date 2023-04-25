use handlebars::{handlebars_helper, Handlebars};
use markdown::{CompileOptions, Options};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};
use thiserror::Error;
use time::Date;
use walkdir::WalkDir;

const STYLE: &str = include_str!("./style.css");
const TEMPLATE: &str = include_str!("./template.html");
const TEMPLATE_NAME: &str = "page";

#[derive(Deserialize, Serialize, Debug)]
struct Metadata {
    title: String,
    description: String,
    date: Date,
    // date: String,
    #[serde(default)]
    tags: Vec<String>,

    #[serde(default)]
    hidden: bool,
}

#[derive(Deserialize, Serialize, Debug)]
struct Page {
    figurative_path: PathBuf,
    absolute_path: PathBuf,

    metadata: Metadata,
    raw: String,
}

#[derive(Debug, Error)]
enum CompilationError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("missing frontmatter")]
    FrontMatter,
    #[error("frontmatter error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("missing content")]
    Content,
    #[error("rendering: {0}")]
    Render(#[from] handlebars::RenderError),
}

fn read_pages(dir: &Path) -> Result<Vec<Page>, CompilationError> {
    let mut pages = vec![];
    let directory = WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file());

    for file in directory {
        let path = file.into_path();
        let page = fs::read_to_string(&path)?;

        let mut figurative_path = path.strip_prefix(dir).unwrap().to_owned();
        figurative_path.set_extension("html");

        let mut split = page.splitn(3, "---\n");
        let metadata = split.nth(1).ok_or(CompilationError::FrontMatter)?;
        let markdown = split.next().ok_or(CompilationError::Content)?;

        let metadata: Metadata = serde_yaml::from_str(metadata)?;

        pages.push(Page {
            absolute_path: path,
            figurative_path,
            metadata,
            raw: markdown.to_owned(),
        });
    }

    Ok(pages)
}

fn hydrate_compile(pages: &[Page], output: &Path) -> Result<(), CompilationError> {
    let mut templating = Handlebars::new();

    handlebars_helper!(crate_link: |name: str, { version: str = "latest" }| {
        format!("[{name}](https://docs.rs/{name}/{version}) ![](/static/crate-small.svg)")
    });

    templating.register_helper("crate", Box::new(crate_link));
    templating
        .register_template_string(TEMPLATE_NAME, TEMPLATE)
        .unwrap();

    fs::remove_dir_all(output).ok();

    let mut sorted: Vec<&Page> = pages.iter().collect();
    sorted.sort_by_key(|&a| std::cmp::Reverse(a.metadata.date));

    for page in pages {
        let output_path = output.join(&page.figurative_path);

        #[derive(Serialize, Debug)]
        struct HydrationData<'a> {
            /// pages sorted by date
            pages: &'a [&'a Page],
            /// current page
            page: &'a Page,
        }

        let options = Options {
            compile: CompileOptions {
                allow_dangerous_html: true,
                gfm_footnote_label: None,
                ..Default::default()
            },
            ..Options::gfm()
        };

        let data = HydrationData {
            pages: &sorted,
            page,
        };
        let hydrated = templating.render_template(&page.raw, &data)?;
        let compiled = markdown::to_html_with_options(&hydrated, &options).unwrap();

        #[derive(Serialize)]
        struct RenderingData<'a> {
            metadata: &'a Metadata,
            content: &'a str,
            style: &'a str,
        }

        let data = RenderingData {
            metadata: &page.metadata,
            content: &compiled,
            style: STYLE,
        };

        fs::create_dir_all(output_path.parent().unwrap())?;
        let mut file = File::create(output_path)?;

        templating.render_to_write(TEMPLATE_NAME, &data, &mut file)?;
    }

    Ok(())
}

fn compile_pages(source: &Path, output: &Path) -> Result<(), CompilationError> {
    let pages = read_pages(source)?;
    hydrate_compile(&pages, output)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    compile_pages("./pages/".as_ref(), "./dist/".as_ref())?;

    Ok(())
}
