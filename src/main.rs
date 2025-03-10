use handlebars::{handlebars_helper, Handlebars};
use markdown::{CompileOptions, Options};
use notify::{RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
    sync::{mpsc, LazyLock, Mutex},
};
use thiserror::Error;
use time::Date;
use walkdir::WalkDir;

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
        .filter(|entry| entry.path().is_file())
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".md"));

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

static INCLUDE_CACHE: LazyLock<Mutex<HashMap<PathBuf, String>>> = LazyLock::new(Default::default);
fn hydrate_compile(pages: &[Page], output: &Path) -> Result<(), CompilationError> {
    let mut templating = Handlebars::new();
    templating.register_escape_fn(handlebars::no_escape);

    handlebars_helper!(crate_link: |name: str, { version: str = "latest" }| {
        format!("[{name}](https://docs.rs/{name}/{version}) ![](/static/crate-small.svg)")
    });

    // let cache: HashMap<String, String> = HashMap::new();

    handlebars_helper!(include: |path: str| {
        let mut cache = INCLUDE_CACHE.lock().unwrap();
        let path = Path::new(path);

        if let Some(file) = cache.get(path) {
            file.to_owned()
        } else {
            let full_path = Path::new("site").join(path);
            let read = std::fs::read_to_string(&full_path).expect("missing included file");

            cache.entry(path.to_owned()).or_insert(read).clone()
        }
    });

    templating.register_helper("crate", Box::new(crate_link));
    templating.register_helper("include", Box::new(include));

    let template = std::fs::read_to_string("./site/template.html")?;
    templating
        .register_template_string("template", template)
        .unwrap();

    // remove previously generated output
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
        let markdown_hydrated = templating.render_template(&page.raw, &data)?;
        let markdown_compiled =
            markdown::to_html_with_options(&markdown_hydrated, &options).unwrap();

        #[derive(Serialize)]
        struct RenderingData<'a> {
            metadata: &'a Metadata,
            content: &'a str,
        }

        let data = RenderingData {
            metadata: &page.metadata,
            content: &markdown_compiled,
        };

        fs::create_dir_all(output_path.parent().unwrap())?;
        let mut file = File::create(output_path)?;

        templating.render_to_write("template", &data, &mut file)?;
    }

    Ok(())
}

fn compile_pages(source: &Path, output: &Path) -> Result<(), CompilationError> {
    let pages = read_pages(source)?;
    hydrate_compile(&pages, output)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let watch = std::env::args().any(|arg| arg == "watch");
    let compile = || compile_pages("./site/".as_ref(), "./dist/".as_ref());

    if !watch {
        compile()?;
        return Ok(());
    }

    // watcher code for development
    let (tx, rx) = mpsc::channel();
    let mut notifier = notify::recommended_watcher(tx)?;

    notifier.watch("./site/".as_ref(), RecursiveMode::Recursive)?;

    println!("Compiling pages and starting watcher...");
    compile()?;

    for event in rx {
        let _ = event?;
        println!("Compiling pages...");
        compile()?;
    }

    Ok(())
}
