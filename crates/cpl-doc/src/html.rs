//! Modules as a static site: `index.html`, one page per module and
//! `style.css`. No scripts to load and no network: the pages open from
//! disk.

use crate::highlight::{escape, highlight};
use crate::markdown;
use crate::{Item, Kind, Module};

/// The stylesheet every page links.
pub const STYLE: &str = include_str!("style.css");

/// Filters the page's items as the reader types. `/` focuses the box.
const FILTER: &str = "\
const box = document.querySelector('.top input');
const rows = [...document.querySelectorAll('[data-name]')];
box.addEventListener('input', () => {
  const want = box.value.trim().toLowerCase();
  for (const row of rows) row.hidden = want !== '' && !row.dataset.name.includes(want);
});
document.addEventListener('keydown', (e) => {
  if (e.key === '/' && document.activeElement !== box) { e.preventDefault(); box.focus(); }
});
";

/// What the pages are called and where their source is.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Options {
    /// The site's name, shown in the top bar and in page titles.
    pub title: String,
    /// When set, every item links to `<source_url>/<path>#L<line>`.
    pub source_url: Option<String>,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            title: "Documentation".to_string(),
            source_url: None,
        }
    }
}

/// The order item kinds are listed in on a module's page.
const GROUPS: [Kind; 9] = [
    Kind::Domain,
    Kind::Type,
    Kind::Struct,
    Kind::Enum,
    Kind::Trait,
    Kind::Impl,
    Kind::Const,
    Kind::Value,
    Kind::Fn,
];

/// The file a module's page is written to. Two modules with the same name
/// (`a/util.cp`, `b/util.cp`) get `util.html` and `util-2.html`.
fn file_names(modules: &[Module]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for module in modules {
        let stem: String = module
            .name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        // `index.html` is the list of modules.
        let stem = if stem == "index" || stem.is_empty() {
            format!("{stem}-module")
        } else {
            stem
        };
        let mut name = format!("{stem}.html");
        let mut n = 2;
        while names.contains(&name) {
            name = format!("{stem}-{n}.html");
            n += 1;
        }
        names.push(name);
    }
    names
}

fn page(options: &Options, title: &str, crumb: &str, side: &str, main: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{title}</title>\n<link rel=\"stylesheet\" href=\"style.css\">\n</head>\n<body>\n\
         <header class=\"top\"><a class=\"brand\" href=\"index.html\">{site}</a>{crumb}\
         <input type=\"search\" placeholder=\"Filter  /\" aria-label=\"Filter this page\"></header>\n\
         <div class=\"layout\">\n<nav class=\"side\">\n{side}</nav>\n<main>\n{main}</main>\n</div>\n\
         <script>\n{FILTER}</script>\n</body>\n</html>\n",
        site = escape(&options.title),
    )
}

/// The sidebar: every module, then the items of the one being read.
fn sidebar(modules: &[Module], files: &[String], here: Option<usize>) -> String {
    let mut out = String::from("<h2>Modules</h2>\n");
    for (i, (module, file)) in modules.iter().zip(files).enumerate() {
        let class = if here == Some(i) {
            " class=\"here\""
        } else {
            ""
        };
        out.push_str(&format!(
            "<a{class} href=\"{file}\">{}</a>\n",
            escape(&module.name)
        ));
    }
    if let Some(module) = here.map(|i| &modules[i]) {
        for kind in GROUPS {
            let items: Vec<&Item> = module
                .items
                .iter()
                .filter(|item| item.kind == kind)
                .collect();
            if items.is_empty() {
                continue;
            }
            out.push_str(&format!("<h2>{}</h2>\n", kind.plural()));
            for item in items {
                out.push_str(&format!(
                    "<a class=\"entry\" href=\"#{}\">{}</a>\n",
                    escape(&item.anchor(None)),
                    escape(&item.name)
                ));
            }
        }
    }
    out
}

fn item_html(module: &Module, item: &Item, owner: Option<&str>, options: &Options) -> String {
    let anchor = item.anchor(owner);
    let class = if owner.is_some() {
        "item member"
    } else {
        "item"
    };
    let mut out = format!(
        "<section class=\"{class}\" id=\"{id}\" data-name=\"{name}\">\n<div class=\"sig\"><span class=\"decl\">{decl}</span>",
        id = escape(&anchor),
        name = escape(&anchor.to_lowercase()),
        decl = highlight(&item.signature),
    );
    if let Some(cost) = &item.cost {
        out.push_str(&format!(
            "<span class=\"cost\" title=\"Time complexity\">{}</span>",
            escape(cost)
        ));
    }
    if let Some(base) = &options.source_url {
        out.push_str(&format!(
            "<a class=\"src\" href=\"{}/{}#L{}\">source</a>",
            escape(base.trim_end_matches('/')),
            escape(module.path.trim_start_matches("./")),
            item.line
        ));
    }
    out.push_str("</div>\n");
    out.push_str(&markdown::render(&item.doc));
    for member in &item.members {
        out.push_str(&item_html(module, member, Some(&item.name), options));
    }
    out.push_str("</section>\n");
    out
}

fn module_page(modules: &[Module], files: &[String], index: usize, options: &Options) -> String {
    let module = &modules[index];
    let mut main = format!(
        "<h1>{}<small>{}</small></h1>\n",
        escape(&module.name),
        escape(&module.path)
    );
    main.push_str(&markdown::render(&module.doc));
    if module.items.is_empty() {
        main.push_str("<p class=\"empty\">This file declares nothing.</p>\n");
    }
    for kind in GROUPS {
        let items: Vec<&Item> = module
            .items
            .iter()
            .filter(|item| item.kind == kind)
            .collect();
        if items.is_empty() {
            continue;
        }
        main.push_str(&format!("<h2 class=\"group\">{}</h2>\n", kind.plural()));
        for item in items {
            main.push_str(&item_html(module, item, None, options));
        }
    }
    let crumb = format!("<span class=\"crumb\">/ {}</span>", escape(&module.name));
    page(
        options,
        &format!("{} · {}", escape(&module.name), escape(&options.title)),
        &crumb,
        &sidebar(modules, files, Some(index)),
        &main,
    )
}

fn index_page(modules: &[Module], files: &[String], options: &Options) -> String {
    let mut main = format!("<h1>{}</h1>\n", escape(&options.title));
    if modules.is_empty() {
        main.push_str("<p class=\"empty\">No .cp files were found.</p>\n");
    }
    for (module, file) in modules.iter().zip(files) {
        main.push_str(&format!(
            "<h2 class=\"group\"><a href=\"{file}\">{}</a></h2>\n",
            escape(&module.name)
        ));
        let summary = crate::model::summary(&module.doc);
        if !summary.is_empty() {
            main.push_str(&format!("<p>{}</p>\n", markdown::render_inline(summary)));
        }
        main.push_str("<table class=\"index\">\n");
        for item in &module.items {
            main.push_str(&format!(
                "<tr data-name=\"{lower}\"><td><a href=\"{file}#{id}\">{name}</a></td><td>{summary}</td></tr>\n",
                lower = escape(&item.name.to_lowercase()),
                id = escape(&item.anchor(None)),
                name = escape(&item.name),
                summary = markdown::render_inline(item.summary()),
            ));
        }
        main.push_str("</table>\n");
    }
    page(
        options,
        &escape(&options.title),
        "",
        &sidebar(modules, files, None),
        &main,
    )
}

/// Every file of the site as `(name, contents)`: `index.html`,
/// `style.css`, then one page per module in the order given.
pub fn render(modules: &[Module], options: &Options) -> Vec<(String, String)> {
    let files = file_names(modules);
    let mut out = vec![
        (
            "index.html".to_string(),
            index_page(modules, &files, options),
        ),
        ("style.css".to_string(), STYLE.to_string()),
    ];
    for (index, file) in files.iter().enumerate() {
        out.push((file.clone(), module_page(modules, &files, index, options)));
    }
    out
}
