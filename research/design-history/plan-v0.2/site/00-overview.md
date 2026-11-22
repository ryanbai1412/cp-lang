# CP Lang website: plan

A from-scratch rebuild of the website, playground, course and docs.
Facts about today's repo are marked *(verified)*; everything else is a
proposal. The new-user story is in `01-journey.md`.

Status: decided in outline (Ryan agreed to the plan, 2026-11-21); the
open decisions in section 8 are still proposed. Where this disagrees
with `../90-docs-and-playground.md`, this document wins.

## 1. Who the site is for

Four readers, in priority order:

1. **A contestant trying CP Lang for the first time.** Comes from C++,
   Python, Java or Rust. Wants to solve a real problem in ten minutes
   without installing anything.
2. **A contestant mid-contest.** Wants one answer quickly ("what does
   `lower_bound` return on an empty range?"): the reference, search and
   the cheat sheet.
3. **A model or person writing code from the docs alone.** This is the
   AGENTS.md "writable from the docs" rule, and today's eval measures it
   (49/55 passing, *verified* in `research/writable-from-docs.json`).
4. **A contributor.** Wants to see how the project is organized: the 14
   domains, the decisions, what is built and what is spec only.

## 2. Principles

- **One source of truth, generated wherever possible.** The stdlib
  reference comes from the `///` docs, the error index from the
  diagnostics registry, CLI pages from the CLI, project pages from
  `plan/DOMAINS.md` and `decisions.md`. Hand-written prose covers only
  what can't be generated.
- **Every snippet is checked.** Every code block on the site goes through
  the real compiler in the gate, the same as today, extended to 0.3
  (section 5).
- **Status is visible.** A beta label across the site, plus a badge on
  every feature that is spec only rather than runnable. Never present a
  spec-only form as if it runs.
- **Static.** Plain files on Cloudflare Pages. The only dynamic piece is a
  tiny Worker for share links (section 6).
- **Modern stack, own design system, no legacy code carried over.** The
  old `site/` is deleted once the new site reaches parity.

## 3. Components

### 3.1 Landing page

The pitch in one screen, then proof.

- Hero: one sentence ("Write the idea, submit fast C++17") and, in the
  same viewport, **the syntax**. The first thing a visitor sees is CP
  Lang code, not copy (Ryan: showcase the syntax as soon as possible).
  It's a live editor with tabs of short programs written for the page
  (not taken from the corpus), each 3 to 15 lines and runnable:
  - *A + B, multitest*: `read`, `print`, the three-line case.
  - *Prefix sums over a domain*: an index space declared once.
  - *DP with `@cache`*: a recurrence that reads like the editorial.
  - *BFS on a grid*: `?T` instead of `-1` sentinels.
  - *Segment tree from the library*: the custom-op case.
  Each tab has *Run*, a *C++17* toggle showing the generated file, and
  a one-line caption saying what to notice. The snippets go through the
  snippet gate like every docs fence, and their status badges show.
  Two buttons sit underneath: *Start the course* and *Open the
  playground*.
- A beta banner with a link to the status page.
- "The bug you can't write": four short C++-vs-CP Lang pairs, each
  removing a P0 bug class (off-by-one with domains, `INF` sentinels with
  `?T`, multitest leakage, overflow with ranges).
- Real ports: two or three side-by-side solutions from the ports gallery,
  with the measured token and runtime ratios from
  `research/scoreboard.ports.jsonl`.
- A "How fast is it?" line linking to the Performance page (3.8): the
  median ratio and the share of ports within 1.1x, both from the bench
  data, never a hand-picked best case.
- The toolchain in one strip: stress, trace, `explain`, judge profiles,
  LSP. Each item links to its tools page.
- Footer: GitHub, status, changelog, feedback.

### 3.2 Beta and status system

- A site-wide dismissible banner: "CP Lang is in beta: the language and
  library can still change. See status."
- A **status page** generated from `plan/DOMAINS.md` (one card per domain
  with its north star, current state and next step) and from `spec/delta/`
  (what the implementation still lacks compared with the spec).
- Per-feature badges in the docs: `runs` (the compiler and playground
  execute it), `checks` (type-checks, can't run yet), `spec` (designed,
  not built). The gate assigns the badge from the fence kind, so the
  badge can't be wrong.

### 3.3 Onboarding: quickstarts by background

One page per background: C++, Python, Java, Rust. Each page is a ten-minute
read built on the same six small problems, so the pages stay parallel and
are cheap to maintain:

1. Read n numbers, print the sum (I/O, `read`, `print`).
2. Two pointers on a sorted array (loops, ranges, slices).
3. BFS on a grid (domains, `?T` distances, no sentinels).
4. A knapsack DP (arrays over domains, `max=`).
5. A multitest problem (per-test state can't leak).
6. A modular combinatorics problem (`Mod<P>`).

Each problem shows "what you'd write in X", "what you write here", and
"the trap X sets that CP Lang removes". The traps differ by background:
C++ gets overflow and UB, Python gets speed and recursion limits, Java
gets I/O boilerplate, Rust gets borrow friction and `as` casts. Every
snippet runs inline. The page ends with a link into the course.

Also under onboarding: *Getting started* (playground first, then install
when it exists) and a *Language tour* (one page, every feature in a few
lines each, for skimmers).

### 3.4 Interactive course

Problem-driven: you learn by solving many CP-style problems in the browser.

- **Structure.** Tracks of units. A unit has a short concept page (half a
  screen of prose, two runnable examples), then 3-8 problems that use the
  concept, easy to hard. Rough units: basics and I/O; arrays and ranges;
  domains and index safety; optionals and relaxation; sorting and
  searching; DP; graphs; trees; range structures (Fenwick, segment tree);
  math and `Mod<P>`; strings; verification (`@brute`, stress, trace).
- **Problems.** Each problem is a directory with a statement, samples,
  hidden tests, a starter, a reference solution, hints in order, and a
  short editorial. A problem is data, so new ones need no code.
- **Judging in the browser.** The WASM compiler runs every test in a
  worker, with a time limit, and shows a per-test verdict (AC, WA, RE,
  TLE) and a diff on WA. The existing write-code exercises already do this
  *(verified: `site/course/*.json`, 10 lessons)*; the new course
  generalizes them.
- **Help when stuck.** Errors link to the error index. "Trace this test"
  opens the trace debugger on the failing input. Hints unlock one at a
  time.
- **Progress** in `localStorage`, with no accounts. Export and import
  progress as a file.
- **A practice set** after the course: the same problem format, filterable
  by tag and difficulty.
- **Problem content must be original.** The corpus statements come from
  real judges, so we shouldn't republish them. Write original problems in
  the same style, or link to the judge. This is the biggest content cost.

### 3.5 Playground

One editor component, reused by the playground page, the course and every
runnable docs snippet.

- CodeMirror 6 editor: CP Lang highlighting, inline diagnostics, the type
  on hover, format on save. These come from the WASM build of the real
  compiler (`crates/cpl-wasm`), which already exposes diagnostics and type
  inspection *(verified, README)*.
- Panes: source, several stdin test cases with expected outputs, stdout,
  generated C++17, `explain` (representations and costs), and trace.
- Run, run all cases, stop, build C++, format, lint.
- Share links (section 6), "open in playground" from any snippet, and an
  embed mode for blog posts.
- **Open a submission**: paste any C++ file `cpl` wrote and the
  playground recovers the CP Lang source from its header comment (the
  header already quotes the full source, *verified*). This backs the
  `/s` page that generated C++ links to (section 3.10).
- An examples menu generated from the cookbook and the ports.
- Runs in a worker with a timeout, as today.

### 3.6 Documentation

Four kinds, kept separate so each stays good at its job:

- **Guides** (learning-oriented, in depth): one page per topic, readable
  front to back. Topics: values and types; numbers, ranges and overflow;
  domains, positions and cuts; optionals; control flow and functions;
  sequences, generators and views; ownership, `mut` and lends; traits and
  generics; I/O and multitest; interaction; modules and local libraries;
  powers (what user libraries can define). Each guide is a readable
  rewrite of the matching `spec/core` chapter, with runnable examples.
- **Language reference** (normative): `spec/core` rendered as is, with
  rule ids as anchors. The site renders the files; it doesn't fork them.
- **Library reference**: one page per module (`seq`, `graph`, `segtree`,
  ...). Every entry shows its signature, summary, and the labelled facts
  (cost, traps, empty, ties, order), plus examples that run inline. It is
  generated from `cpl doc --format json` with `CPL_STD=spec/std`, so the
  stub docs are the single source. Above the entries, each module page
  starts with an overview from `spec/lib/<chapter>.md` (the model and when
  to use it).
- **Cookbook**: "how do I..." recipes by task (compress coordinates, binary
  search the answer, read a tree from parents, print a grid, ...), each a
  complete runnable program.
- **Error index**: one page per diagnostic and lint, generated from `cpl
  explain --json` as today *(verified, `site/errors.py`)*, with a broken
  example and its fix.
- **Cheat sheet**: one printable page, generated from or checked against
  the reference.
- **Glossary**: domain, cut, position, lends, niche, power, judge profile,
  ... Each term links to its guide.

### 3.7 Tools

- **CLI reference**, generated from the CLI's help text. It covers `check`,
  `run`, `build`, `judge`, `judges`, `submit`, `fmt`, `lint`, `explain`,
  `doc`, `new`, `test`, `stress`, `bench`, `profile`, `cover`, `watch`,
  `lsp`, `dap` *(verified from `cpl --help`)*.
- **Workflow guides**: the contest loop (`new`, `watch`, `test`, `submit`),
  stress testing and shrinking, the trace debugger and time travel,
  interactors and checkers, judge profiles and the 64 KB limit, profiling
  and `explain`.
- **Editor setup**: VS Code, Neovim, Helix, Zed, Emacs and JetBrains, which
  all exist under `editors/` *(verified)*.
- **The trace viewer** as a standalone page, as today.

### 3.8 The project

- **Design**: the mission and the goals from AGENTS.md, written for users.
  "Why not just C++?" lives here.
- **Domains**: the 14 areas, generated from `plan/DOMAINS.md` (shared with
  the status page).
- **Decisions**: a searchable index of `decisions.md`, one anchor per
  decision, generated.
- **Real problems** (the corpus gallery, replacing today's ports
  gallery): the 100 hill-climb problems from `corpus/`. Each shows the
  original accepted C++ next to `spec.cp`, the notes and tags, the
  measured ratios, and a link to the judge's statement. It's generated
  from `corpus/index.json` and `meta.json`, so it costs nothing to
  maintain. This is a main pillar for persuading contestants, but not
  the home page hero. The parts that need care:
  - The submissions are Ryan's and Andrew's own code, so we need both of
    your OKs to publish them. We link to judges' statements and never
    copy them.
  - We show only records whose `spec` status is `reviewed` or `gold`.
    Each gets a status badge: most `spec.cp` files `check` but can't run
    yet, so we don't present them as runnable until they are.
  - `dream.cp` uses syntax that doesn't exist, so it stays off the page,
    or appears only on Design as "what we wished for", clearly labelled.
  - The notes cite decision numbers, so we link each one to the
    decisions index and rewrite internal jargon at build time where
    needed.
  - Course problems link here ("see a real problem using this"), which
    replaces some hand-written course examples.
- **Performance**: speed and size of the generated C++ against matched
  C++ originals, generated from `bench/` (`corpus-p95-results.json`,
  the latency audit), never typed by hand. It shows the whole
  distribution, not a cherry-picked number: runtime median, p90, p95,
  worst; peak memory; largest emitted file against the 64 KB limit;
  front-end latency against the 500 ms budget; the 20 slowest ports with
  links; the run's commit, compiler and machine. Today's committed run
  *(verified)* is 343 ports with median 0.98x but p95 2.61x against the
  1.1x target, and the largest file is 33.6 KB. The page has to say
  "target not met" until it is, and the landing page links to this page
  instead of quoting a single number.
- **Changelog and roadmap**: per release, generated from a `CHANGELOG.md`
  we'd start keeping.
- **Contributing**: how to run the gate, the commit conventions, where to
  start.

### 3.9 Get set up: install, editors, local loop

Moving people from the browser to their own machine is its own onboarding
step, with one **Setup** page shaped as a checklist (each step has a
checkbox, kept in `localStorage`, and a "verify" command):

1. **Install `cpl`.** Prebuilt binaries per platform plus a one-line
   installer script; later Homebrew, Scoop and `cargo install`. Today
   the only path is `cargo install --path crates/cpl-cli` *(verified,
   `editors/README.md`)*. Include upgrade and uninstall.
2. **Install the editor extension.** One tab per editor, each with the
   install step and a 20-second GIF of diagnostics, hover and the Run/
   Trace/Test/Stress lenses:
   - VS Code: one click from the Marketplace, plus Open VSX for Cursor,
     Windsurf and VSCodium. The extension exists *(verified:
     `editors/vscode`, publisher `cp-lang`, v0.2.2)* but isn't
     published yet.
   - Neovim, Helix, Zed, Emacs, JetBrains: these setups already exist
     under `editors/` *(verified)*. Each gets a copy-paste snippet.
   - **Scope (Ryan): design for it, don't build it yet.** The page and
     its tabs are designed for one-click installs, but until the
     extensions are published each tab shows today's from-source
     install (`editors/README.md`). Switching a tab to the Marketplace /
     Open VSX / registry button later is a content change, not a
     redesign.
   - One "Other editor" section: any LSP client with `cpl lsp`.
3. **Check it works.** A `cpl doctor` command (new; it doesn't exist
   today) that checks the binary, the version, `g++`, the judge profile,
   and whether an editor is talking to `cpl lsp`. The page shows the
   expected output.
4. **Pick your judge.** `cpl judges` and setting the default profile
   (Codeforces, AtCoder, ...), with the reason it matters (flags, stack
   size, the 64 KB limit).
5. **Set up the contest loop.** `cpl new` from samples, `cpl watch`, `cpl
   test`, `cpl submit`. Also the **Competitive Companion** browser
   extension (it sends a judge's problem and samples to a local port), so
   one click on a problem page creates the problem with its tests. That
   needs a small `cpl` listener (new).
6. **Optional extras**: shell completions, a personal library folder (the
   local-library setup the docs already show), and an AI-assistant setup
   (a rules snippet for Cursor/Copilot/Claude that points at `/llms.txt`).

Places that nudge users toward setup, without nagging:

- After the first AC in the course: "Solving in your editor is faster:
  get the extension (2 min)".
- In the playground header: a quiet "Run locally" link that opens Setup.
- At the end of every quickstart.
- In the footer of every docs page: the VS Code button.

Also on the page: a short **"Submitting to a judge"** guide covering
which judges accept the generated C++17, which compiler to pick on each,
and what to do if a judge rejects the file.

### 3.10 Components you didn't list

- **Search**: full text over everything, with keyboard-first `/` and
  Ctrl+K. Fumadocs builds a static index at build time, with no server.
- **LLM-friendly docs**: `/llms.txt` plus one generated Markdown file with
  the whole guide, reference and library. This turns "writable from the
  docs" into a product feature, and the eval can read exactly this file.
- **Versioned docs**: a version switcher (0.3 now; older versions frozen
  later) so old solutions keep matching their docs.
- **Feedback on every page**: "Edit this page" and "Report a docs bug"
  links to GitHub.
- **Basics**: dark mode, a 404 page, OpenGraph images (generated per page,
  showing a code snippet), sitemap, accessibility (keyboard, contrast,
  reduced motion), and a performance budget.
- **Analytics**: none, or Cloudflare Web Analytics (cookieless). I'd start
  with none.
- **Offline single-file edition**: today's `dist/cp-lang.html`. I'd drop it
  for now (see open decision 5).
- **Staying in touch**: release notes (a launch item, Ryan) as an RSS
  feed, a "what's new in 0.x" page per release, a GitHub star / watch
  prompt, and a community link (Discord or GitHub Discussions) once one
  exists.
- **Printable cheat sheet**: a print stylesheet and PDF, for onsite
  contests that allow paper.
- **Growth loops** (the two ways people actually hear about a CP
  language):
  - *Submissions advertise.* Generated C++ gets one more header line,
    `// Written in CP Lang: <site>/s` (a `cpl` codegen change, new; one
    flag or judge-profile setting turns it off). `/s` explains what the
    reader just saw and offers "Open a submission" (section 3.5). The
    link never points at an uploaded copy of the source: nothing is
    published that the submission didn't already contain.
  - *Friends share.* *Share* on a course problem offers "my solution" or
    "challenge a friend" (the problem page with "X solved this" and no
    spoilers). Every problem page must work cold, with a collapsible
    two-minute intro. Shared links unfurl with an OpenGraph card showing
    a code snippet; see section 6 for why that needs a short link.

## 4. Design system: copied, not built

Ryan: no custom design system; copy something beautiful and proven.

- **shadcn/ui, used as published**: its default neutral theme, its CSS
  variables, Radix-based components, Geist (or Inter) and Geist Mono. We
  add only one thing on top: a status badge variant (`runs`, `checks`,
  `spec`), built from existing shadcn Badge variants. We don't change any
  tokens.
- **Fumadocs for the docs chrome.** It is built on shadcn's tokens and
  Tailwind and already includes the sidebar, TOC, search dialog,
  callouts, tabs, code blocks with titles and copy, steps, cards and
  previous/next. Docs pages get those for free. Landing, playground and
  course pages use shadcn blocks.
- What we still write ourselves are the product widgets that don't exist
  anywhere: the editor, the test case panel, verdicts, C++ vs CP Lang
  compare, the API entry. All of them are assembled from shadcn
  primitives, not new styles.
- If you'd rather copy a specific site's look instead (the Vercel/Geist
  docs, Stripe docs, the Rust book), it slots in the same way. I'd still
  start from shadcn because its components can be copied whole.

## 5. Architecture

### 5.1 Stack

Changed from the first draft (Astro) so we can use Fumadocs and shadcn
directly:

- **Next.js** (App Router) with `output: "export"`: plain static files,
  no server at runtime. TypeScript in strict mode.
- **Fumadocs** (MDX content, docs layout, static search index) and
  **shadcn/ui + Tailwind v4**.
- **Shiki** for highlighting, using the existing
  `editors/vscode/syntaxes/cp.tmLanguage.json` *(verified)*, so the
  grammar has one source.
- **CodeMirror 6** for editing.
- **pnpm**, Vitest for unit tests, Playwright for end-to-end tests.
- Lives in a new top-level `web/`; `site/` stays until cutover.

Alternatives: Astro + Starlight (lighter output, but its look isn't
shadcn, and shadcn works less naturally there), VitePress (Vue, no
shadcn).

### 5.2 Generated content

Build-time loaders in TypeScript that call the real `cpl` binary and the
repo files. No second compiler and no regex scraping of Rust:

```
stdlib    <- CPL_STD=spec/std cpl doc --format json stdlib
errors    <- cpl explain --json
cli       <- cpl <cmd> --help   (or a new `cpl help --json`)
domains   <- plan/DOMAINS.md
decisions <- decisions.md
ports     <- research/scoreboard.ports.jsonl + proto/ports/**
corpus    <- corpus/index.json + corpus/problems/*/{meta.json,spec.cp}
perf      <- bench/corpus-p95-results.json + latency audit JSON
course    <- web/content/problems/<id>/{problem.md,tests/,ref.cp,...}
spec      <- spec/core/*.md, spec/lib/*.md   (rendered, not copied)
```

This reuses `cpl doc --format json` rather than writing a separate HTML
generator for `spec/std`. You declined a separate generator earlier, so
confirm this is acceptable (open decision 3).

### 5.3 The snippet gate

Every fenced block declares how it is checked, and the gate enforces it:

````
```cp            runs: cpl check + cpl run (+ expected output if given)
                 + cpl build + g++ -fsyntax-only, as today
```cp check      type-checks against today's compiler; can't run yet
```cp spec       CPL_SPEC=1 CPL_STD=spec/std cpl check
```cp error E0204  must fail with exactly that code
````

The fence kind drives the badge readers see. Library `example:` lines
become generated tests. The gate also checks links and anchors, runs
Playwright smoke tests (the playground runs, a course problem gets AC,
search finds a term) and keeps a size budget. It hooks into
`scripts/check.sh --fast`, which then needs Node and pnpm in the fast gate
(today Node is needed only for `--full`, *verified*).

### 5.4 WASM

Reuse `scripts/build-wasm.sh` and `crates/cpl-wasm` unchanged. The site
loads the module once in a worker shared by every editor on the page.
Today's gzipped module is 1.3 MB *(verified from the build)*, so load it
lazily on the first run, not on page load.

## 6. Hosting and share links

- **Cloudflare Pages** serves the static build, with preview deploys per
  branch.
- **Share links**:
  - Short programs go in the URL fragment, compressed
    (`/play#c=<deflate+base64url>`). No server, nothing stored, works
    offline, private by default. Most programs fit.
  - Longer programs (or when the URL gets too long): "Create short link"
    POSTs to a ~50-line Cloudflare Worker. The Worker stores the snapshot
    (source, test cases, compiler version) in R2 under its content hash and
    returns `/p/<hash>`. Snapshots are immutable and capped in size, with
    no accounts.
  - I'd use R2 rather than S3: it speaks the S3 API, has no egress fees,
    and runs on the same platform as the site. A static site can't write
    to S3 without some server to sign uploads anyway.
  - Abuse controls: size cap, a per-IP rate limit, optionally Turnstile.
  - Link previews: chat apps never send the URL fragment to the server,
    so a fragment link can only unfurl as a generic card. *Share* for
    chat therefore makes a `/p/<hash>` link, and the Worker serves an
    OpenGraph image of the code for it. Fragment links stay the private
    option.
- Setup you'd need to do later: a Cloudflare account, a Pages project, an
  R2 bucket, the domain. Until then I build and test everything locally
  and deploy nothing.

## 7. Phases

Each phase lands as small, gated commits on a branch cut from the `spec` branch.

0. **Foundations**: the `web/` skeleton with Fumadocs and shadcn as
   published, the WASM worker, the snippet gate wired into `check.sh`, and
   drafts of three key pages for your review.
1. **Playground + landing + quickstarts**: the editor component, the full
   playground, fragment share links, the landing page, the four
   quickstarts, getting started, the tour.
2. **Reference**: the generated library reference, error index, CLI
   reference, and spec rendering; existing guides migrated and moved to
   0.3; search; the cheat sheet.
3. **Course**: the problem format, the runner and verdicts, the first
   tracks with original problems (content-heavy, continues afterwards),
   practice set.
4. **Project, setup and launch**: status, domains, decisions, ports,
   changelog, release notes (RSS and a "what's new" page per release),
   llms.txt, the Setup page with from-source extension installs (adding
   `cpl doctor` is outside the site), the share-link Worker, Cloudflare
   config, OG images. Publishing the extensions comes later.
5. **Cutover**: point the writable-from-docs eval at the new docs, check
   the pass rate doesn't drop, move the gate over, delete `site/`.

## 8. Open decisions (with my recommendation)

1. **Which language does the site teach?** Recommend 0.3 (the spec),
   because that is what the site launches with. Every snippet carries a
   status badge, and the course uses only forms that run today, growing as
   the implementation lands.
2. **Stack.** Next.js static export + Fumadocs + shadcn/ui, as above.
3. **The library reference generated from `spec/std` via `cpl doc --format
   json`.** Recommend yes. It is rendered by site components, with no
   separate generator.
4. **Share links.** Recommend URL fragments first, plus the Worker + R2 for
   long programs. Say if you want S3 specifically.
5. **The offline single-file edition.** Recommend dropping it until
   there's demand.
6. **Course problems.** Recommend original problems only, never copied
   judge statements.
7. **Node + pnpm in the fast gate.** Recommend yes; the site can't be
   checked without them.
8. ~~Publishing the editor extensions~~ **Decided (Ryan):** design the
   Setup page for one-click installs, don't publish yet. Publishing needs
   publisher accounts you own and prebuilt `cpl` binaries (the
   extensions need `cpl` on the PATH).
