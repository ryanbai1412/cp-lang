# CP Lang field guide

A static, responsive documentation site. No frontend build chain, CDN,
analytics, or runtime dependency. Python 3.10+ and Mistune 3 generate plain
HTML; all links work under a subdirectory as well as at the server root.

## Build and preview

From the repository root:

```sh
python3 -m venv site/.venv
site/.venv/bin/pip install -r site/requirements.txt
site/.venv/bin/python site/build.py
python3 -m http.server 8000 --bind 0.0.0.0 --directory site/_build
```

Open <http://localhost:8000>. Serve the generated directory over HTTP for
search and clipboard support; do not open the files with `file://`.
`_build/` is generated and ignored. No deployment is configured or required.

## Check

```sh
site/.venv/bin/python site/test_site.py
```

This builds every page, validates local links and heading fragments, and
compiles all `example` snippets with the Python prototype and `g++` C++17.
Eleven sample/edge-case runs check actual output. Compiler APIs avoid the
prototype CLI's source-stamping side effect. Design blocks are explicitly
labeled and are not compiled; they must be reviewed against the design.
The full Rust workspace gate remains `cargo test` before a push.

## Edit

- `content/` contains the authored Markdown. `build.py` defines chapter order
  and metadata. H2 headings become the page's table of contents.
- A fenced `example NAME` block embeds `examples/NAME.cp`; add its input and
  expected output to `test_site.py`. Use `cp` for approved-design fragments,
  with an explicit label if a different status applies.
- `template.html` and `assets/` own the shared layout, colors, behavior, and
  local font. Markdown raw HTML is escaped by default.
- Search indexes all eight chapters; `/` or Ctrl/Cmd+K opens it. Arrow keys
  navigate results, Enter follows, and Escape closes. Themes persist locally.

## Content contract

This guide distinguishes the approved language from what is implemented in
the Rust frontend and Python prototype. Never present a design-only snippet
or a planned CLI command as a successful run. The unified-index/projection
design is specified in `plan/19-index-and-lends.md` and `decisions.md`;
locked updates are `let K = n`, `.l`/`.r`, ASCII-only text,
Euclidean `/` and `%` (`//` is a comment), and explicit `Bitset<D>`.
The term input *spec* currently refers to typed reads and constraints, not
an invented `spec` keyword. Keep these labels current as the compiler lands.

Bundled DejaVu Sans Mono is covered by `assets/FONT-LICENSE.txt`.
