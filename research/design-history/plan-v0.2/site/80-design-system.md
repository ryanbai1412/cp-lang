# Design system: shadcn/ui and Fumadocs, as published

Status: decided (Ryan: copy something beautiful and proven; don't build
our own).

Ryan, 2026-11-21: "all the pages should look unified". Every page,
including the playground, Learn, the corpus and the generated
reference, uses the same header, layout, type scale and components. A
page that needs its own chrome is a bug.

The site's library pages and the HTML that `cpl doc` writes should look
more or less the same. The site sets the bar: `cpl doc` is raised to
it, the site is never simplified to match. Until then the site renders
`cpl doc --format json` itself; what each item shows (signature, cost,
traps, examples, source link) is the target for `cpl doc`'s own HTML.

## What we use

- **shadcn/ui** components copied in with its CLI, default **neutral**
  theme, its CSS variables unchanged, light and dark. Radix primitives
  underneath.
- **Tailwind CSS v4**.
- **Geist** and **Geist Mono** fonts.
- **Fumadocs UI** for every docs page: layout, sidebar, TOC, search
  dialog, callouts, tabs, steps, cards, code blocks with title and copy,
  previous/next.
- **lucide** icons (shadcn's default).

The only addition: a status badge with three variants (`runs`,
`checks`, `spec`) built from shadcn's Badge variants (`default`,
`secondary`, `outline`). No new tokens, colours, spacing or radii.

## Page templates

- **Marketing** (landing, `/s`, `/design`): shadcn blocks for hero,
  feature grid, cards, footer.
- **Docs** (guides, reference, lookup, tools, project): Fumadocs
  `DocsLayout`.
- **App** (playground, course problem, trace): a full-height shadcn
  `ResizablePanelGroup` with a top bar.

## Product widgets (assembled from shadcn parts, no new styling)

- **Editor**: CodeMirror inside a Card, toolbar of Buttons, Tabs for
  panes.
- **Test-case panel**: Tabs of Textareas with a pass/fail Badge each.
- **Verdict**: an Alert with the verdict's Badge, and a diff view for
  WA.
- **Compare**: two code blocks side by side (Fumadocs code block), used
  for C++ vs CP Lang everywhere.
- **API entry**: signature code block, a definition list for the
  labelled facts, examples as runnable fences.

## Copy style

- Plain words, short sentences, second person.
- One term per concept, from `spec/GLOSSARY.md`; never its "never"
  column.
- Every claim about speed, size or safety links to its evidence page.
- "Beta" said once per page at most (the banner).
