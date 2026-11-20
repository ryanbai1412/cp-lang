# CP Lang course and documentation

A static, responsive course, reference guide, and example playground. No frontend build chain, CDN,
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

Open <http://localhost:8000>. Keyboard search also works with `file://`;
clipboard support depends on browser permissions.
`_build/` is generated and ignored. No deployment is configured or required.

## Check

```sh
site/.venv/bin/python site/test_site.py
node --test site/test_course.cjs
```

This builds every page, validates local links and heading fragments, and
compiles all `example` snippets and the foundations' worked programs with the
Python prototype and `g++` C++17.
Sample/edge cases and every course prediction check actual native output
against the browser example runner. Compiler APIs avoid the
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
  system font stacks. Markdown raw HTML is escaped by default.
- `course/` stores six progressive lessons' metadata and exercise banks.
  Lesson prose is in `content/course-*.md`. An empty fenced `exercise ID`
  block renders a checked knowledge or output-prediction exercise with a
  hint and solution. `course.py` validates IDs, references, and answer data
  at build time. Exercise prose is escaped, never treated as HTML.
- Search indexes the entire course and reference; `/` or Ctrl/Cmd+K opens it. Arrow keys
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

## Offline edition

```sh
python3 site/bundle.py
# Open dist/cp-lang.html directly in a browser, including offline.
```

The single HTML includes the complete course and reference, full-text keyboard search, an
interactive example playground, and the complete trace debugger. It stores
shared CSS, scripts, and the search index once, with no inline font blobs
or automatic network requests. Hash routes support bookmarks and history.
External repository links still require a connection.

The playground runs explicit JavaScript implementations of the six
documented examples, checked against the real Python-to-C++ prototype in
`test_site.py`. It is not an arbitrary-source compiler: the source is read
only, input is editable, and the UI states this limitation.

Course checks do not pretend to compile arbitrary source: conceptual questions
check a selected answer, and output exercises compare predictions to native-tested
fixtures, ignoring whitespace but not numeric precision or token order. Solutions
are intentionally available without a login. Correct answers and chapter progress
persist in browser storage; reset clears an exercise, changing an answer requires a
fresh check, and changed exercise definitions invalidate old completion records.
If storage is blocked, the course remains usable with page-local progress.
The offline bundle uses exactly the same exercise engine, without fetches.

Both surfaces use the MIT-licensed [shadcn/ui zinc palette](https://ui.shadcn.com/docs/theming)
and a [VitePress-style docs layout](https://vitepress.dev/), implemented as
plain CSS without a framework dependency. White/zinc surfaces, blue
interaction states, 1px borders, and system sans/monospace stacks are shared
across the docs, runner, and trace; red in the trace means a mismatch/error.
The unused historical local font is not embedded or requested by the site.
