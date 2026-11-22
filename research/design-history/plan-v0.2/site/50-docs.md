# Docs (`/docs`)

Status: proposed. Fumadocs layout: sidebar, page, TOC, search. Four
kinds of page, kept apart: **guides** (how to do X), **reference** (what
exactly X is), **lookup** (cookbook, errors, cheat sheet, glossary) and
**the spec itself**. Every page has "Edit this page", "Report a docs
bug", a "last changed" date from git, and previous/next.

## Sidebar

```
Guides
  Thinking in CP Lang
  Input and output
  Domains and indices
  Optionals and sentinels
  Numbers, ranges and overflow
  Strings
  Collections and sequences
  Ownership and mutation
  Functions, closures and generators
  Structs, enums and matching
  Operations, folds and range structures
  Graphs and trees
  Dynamic programming
  Testing: brute, stress, checkers
  Interactive problems
  Performance and control
  Your own library
Reference
  Language (spec/core, rendered)
  Library (spec/std, generated)
  Library families (spec/lib, rendered)
  Tools (spec/tools, rendered)
Lookup
  Cookbook
  Errors
  Cheat sheet
  Glossary
```

## Guides

Each guide is hand-written MDX, 1,500 to 3,000 words, organized around
contest tasks, with runnable fences and links into the reference. The
existing pages are the starting material (*verified* in `site/content`):
`io-and-formatting.md`, `indices.md`, `strings.md`, `borrowing.md`,
`types-and-control.md`, `idioms.md`, `testing.md`, `cpp-guide.md`.
They are migrated to 0.3 spellings (`None`/`Some`, `@cache`, `fold(r)`,
`s.search(p)`, glossary terms) and split or merged into the list above.
Outline of each guide:

- **Thinking in CP Lang**: rethink, don't translate; one fact in one
  place; types carry the statement; costs are visible. From
  `cpp-guide.md` and `idioms.md`.
- **Input and output**: `read` with types and ranges, tuples and
  records, arrays over domains, `read_all(until:)`, multitest, printing
  with separators, 1-based input, fast I/O defaults. Spec 10.
- **Domains and indices**: `domain`, positions, cuts, ranges,
  product domains, `Pair<V>`, `pieces`, keys of slices. Spec 4, 4b.
- **Optionals and sentinels**: `?T`, `None`/`Some`, `??`, `!`,
  `min=`/`max=`, niches. Spec 2.4, 3.
- **Numbers, ranges and overflow**: range types, widths, overflow
  proofs, `Mod<P>`, floats. Spec 3.
- **Strings**: `Str`, chars, `search`/`rsearch`, `find` with predicates,
  string algorithms. `lib/strings.md`.
- **Collections and sequences**: lists, maps, sets, bags, deques,
  heaps; generators, views, adapters, `collect`, `sorted`, `buckets`,
  `group`. Spec 7, `lib/containers.md`, `lib/sequence-patterns.md`.
- **Ownership and mutation**: owned and lent, `mut`, `.clone()`,
  `-> lends T`, why no lifetimes. Spec 5.
- **Functions, closures and generators**: nested fns, lambdas,
  tuple-unpacking lambdas, generators, decorators, `@cache`. Spec 6.
- **Structs, enums and matching**: Spec 2.6, 2.7.
- **Operations, folds and range structures**: operation values and laws,
  `SegTree`, `LazySegTree`, `Fenwick`, `Sparse`, windows, `fold(r)`.
  `lib/ops-ranges.md`.
- **Graphs and trees**: `Graph`, arcs and edges, BFS/DFS, `dijkstra`,
  `closure(over:)`, flows, `Rooted`. `lib/graphs-trees.md`,
  `lib/flows.md`.
- **Dynamic programming**: tables over domains, `@cache`, speed-ups.
  `lib/search-dp.md`.
- **Testing**: `@brute`, `cpl stress`, shrinking, generators,
  `@checker`, laws. `lib/random-stress.md`, `tools/stress.md`.
- **Interactive problems**: `@!interactive`, `wire.ask`,
  `@interactor`, flushing. Spec 10.4.
- **Performance and control**: `cpl explain`, release vs checked
  builds, layouts, raw arrays, inline C++. Spec 9, 13.
- **Your own library**: library files, `use`, pins, publishing to your
  own folder. Spec 11.

## Reference

### Language: `/docs/ref/core/<chapter>`

`spec/core/*.md` rendered as-is (never copied into `web/`): one page
per chapter, every `##` a stable anchor, rule ids (R4, R9...) as
anchors. A banner on each page: "This is the normative specification.
For a gentler introduction see <guide>." Chapters with design notes
(`04a-identity-vs-offset.md`, the "Open" sections) render those parts
in a collapsed "Design notes" block.

### Library: `/docs/ref/std/<module>`

Generated from `CPL_STD=spec/std cpl doc --format json stdlib` (no
separate generator). One page per module (`segtree`, `graph`, `seq`,
... 39 modules, *verified*), each with a summary and every public item:

- signature (highlighted, types linked);
- the summary paragraph;
- the labelled facts as a definition list: **cost**, **traps**,
  **empty**, **ties**, **order** (the doc standard in
  `spec/std/README.md`, *verified*);
- examples, runnable when their status is `runs`;
- "Used in": links to cookbook entries and real problems that call it
  (computed from the corpus and cookbook at build time).

An A to Z index page lists every function with its module and cost, and
search indexes every signature.

### Library families and tools

`spec/lib/*.md` and `spec/tools/*.md` rendered like the core chapters,
under "Library families" and "Tools".

## Lookup

- **Cookbook** (`/docs/cookbook/<recipe>`): today's six categories and
  their recipes (`site/cookbook.py`, *verified*), each recipe a complete
  program with a short "why it's written this way". Recipes move from
  `cookbook.py` into `web/content/cookbook/*.mdx`; the program comes from
  the port or corpus file named in front matter.
- **Errors** (`/docs/errors/<code>`): one page per code, generated from
  `cpl explain --json` (title, summary, explanation, trigger, fix, all
  gated). The compiler's diagnostic links here.
- **Cheat sheet** (`/docs/cheat-sheet`): one page, two columns, every
  common form; generated from `docs/one-page-cheat-sheet.md` moved to
  0.3. A print stylesheet makes it a two-page PDF.
- **Glossary** (`/docs/glossary`): rendered from `spec/GLOSSARY.md`
  (the "never" column shown as "don't say"). Every glossary term in a
  guide links here on first use.

## Search

Fumadocs' static index (Orama) over guides, reference, library
signatures, errors, cookbook and course problems; Ctrl+K and `/`.
Results grouped by kind. A library search result shows the signature and
cost inline.

## Docs for models: `/llms.txt` and `/llms-full.txt`

- `/llms.txt`: what CP Lang is and an index of the Markdown versions of
  every page (`/docs/<page>.md`).
- `/llms-full.txt`: one Markdown file with the guides, the cheat sheet,
  the language reference and the library reference, in that order.
- `scripts/writable-from-docs.py` switches its `DOC_FILES` bundle to
  `/llms-full.txt` at cutover, so the eval measures exactly what models
  get.

## Versioning

The site documents the current spec version (0.3). When 0.4 ships, 0.3
is frozen under `/v0.3/`. A version badge in the header.
