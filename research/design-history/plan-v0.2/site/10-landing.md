# Landing page (`/`)

Status: proposed. One screen of "what it looks like" and "why", then
proof, then where to go. The first thing a visitor sees is CP Lang code
(Ryan: "on the home page we just need to showcase the syntax asap").

## Value proposition (Ryan, 2026-11-21)

Three pillars, in this order, each with its own section and evidence:

1. Clean, expressive syntax: hero tabs of short 0.3 programs (`checks`).
2. A powerful standard library: whole solutions to real olympiad
   problems, lifted from `corpus/problems/<id>/spec.cp` (`checks`).
3. Easy debugging: a planted bug, real `cpl stress --trace --explain`
   output regenerated on every build, and the trace viewer on the shrunk
   case (`runs`, today's compiler).

"Bugs you can't write" and "submit with confidence" are supporting
lines, not pillars; the sections below that lead with them are
superseded by this list.

## Order of sections

1. Beta banner
2. Hero with the syntax showcase
3. Where to start
4. The bugs you can't write
5. How fast is it
6. Real problems
7. The toolchain
8. Footer

## 1. Beta banner (site-wide)

One line, dismissible (dismissal stored in `localStorage`, and shown
again when the minor version changes):

> CP Lang is in beta: the language and library can still change.
> [What works today ->](/status)

## 2. Hero with the syntax showcase

Left column on wide screens, above the code on narrow ones:

> **Write the idea. Submit fast C++17.**
>
> CP Lang is a language for competitive programming. Types carry what
> the statement tells you, common bugs can't be written, and every
> program compiles to one readable C++17 file you submit anywhere.
>
> [Start with a problem] [Open the playground]

Right column: the showcase editor (the shared editor component from
`40-playground.md` in its compact mode). Five tabs, each a short program
written for this page. Each tab has *Run* (stdin prefilled), a *C++17*
toggle that swaps the editor for the generated file, a status badge, and
a caption of one sentence saying what to notice. The editor is editable;
*Open in playground* carries the edited program over.

Rules for the snippets:

- Written for the page, not taken from the corpus or ports.
- 3 to 12 lines, no comments needed to follow them.
- Each one shows something C++ can't say as directly.
- Each lives in `web/content/landing/<tab>.cp` with `<tab>.in` and
  `<tab>.out`, and goes through the snippet gate.
- At launch the hero shows only `runs` snippets. A tab whose 0.3 form
  doesn't run yet keeps a runnable form until it does; the 0.3 form sits
  beside it as `<tab>.spec.cp` and replaces it when the gate says it
  runs.

### Tab 1: "Multitest" *(runs today, verified)*

```
@!multitest
let (a, b): (Int, Int) = read()
print(a + b)
```

Input `3 / 1 2 / 10 20 / -5 5` gives `3 / 30 / 0`. Caption: "The test
count is read for you, and nothing leaks from one test to the next."

### Tab 2: "No sentinels" *(runs today, verified)*

```
let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]Int = read()

var best: ?Int = None
for x in a:
    if x % 2 == 0:
        best max= x
print(best ?? -1)
```

Caption: "`?Int` is 'maybe no answer'. No `-1` or `INF` to forget, and
usually no extra bytes."

### Tab 3: "Index spaces" *(0.3 form checks; runnable form used until it runs)*

```
let (n, q): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let a: [V]1..=1e9 = read()
let pre = a.scan(+)
for _ in 0..q:
    let (l, r): (V, V) = read()
    print(pre.fold(l..=r))
```

Caption: "Positions of `V` can't be mixed with other indices, the input
is validated against the statement, and the sum's width is proven to
fit."

### Tab 4: "DP that reads like the editorial" *(checks; `@cache` doesn't run yet, verified)*

```
let n: 1..=90 = read()

@cache
fn fib(k: 0..=90) -> Int:
    if k <= 1:
        return k
    fib(k - 1) + fib(k - 2)

print(fib(n))
```

At launch this tab uses a real DP (coins or LIS) in whatever form runs,
and switches to `@cache` when it runs. Caption: "Memoized recursion with
the table sized from the argument's range."

### Tab 5: "Shortest paths" *(checks under the spec; doesn't run yet, verified)*

```
domain V = 1..=read<1..=2e5>()
let m = read<0..=2e5>()
let g = Graph.undirected(read<[m](u: V, v: V, w: 1..=1e9)>())
let p = dijkstra(g, from: [(V.first!, 0)])
print(p.dist[V.last!] ?? -1)
```

This is the program from `spec/core/00-overview.md`. Caption: "Five lines:
the vertex count bounds the storage, weights are validated, an
unreachable vertex is `None`."

## 3. Where to start

Three cards under the hero:

- **Solve your first problem** (2 minutes): opens course problem 1.
- **Coming from...** C++ · Python · Java · Rust: the four quickstarts.
- **Look something up**: the docs, with the search box inline.

## 4. The bugs you can't write

Heading: "The bugs you can't write". Four pairs. Each pair is the C++
on the left with the bug highlighted and a one-line note of what goes
wrong, and the CP Lang on the right with a one-line note of why it can't
happen. Each pair links to the guide that explains it. Sources:
`web/content/landing/bugs/<n>.{cpp,cp}`, both gated (the C++ is compiled
with g++ and shown to produce the wrong answer on the given input).

1. **Multitest state.** C++: a global `cnt[]` not cleared between tests.
   CP Lang: state declared after `@!multitest` is per test.
2. **Sentinels.** C++: `dist = INF` and `INF + w` overflows, or `-1`
   printed when it shouldn't be. CP Lang: `?Int`, `??`, `min=`.
3. **Mixed-up indices.** C++: `adj[e]` where `adj[v]` was meant, both
   `int`. CP Lang: `[V]` and `[E]` are different types; the mix is a
   compile error naming both.
4. **Overflow.** C++: `int` product of two values up to `1e9`. CP Lang:
   range types prove the width, or the compiler asks for a wider type.

## 5. How fast is it

One line and a link, numbers from `bench/corpus-p95-results.json` at
build time:

> On 343 real problems the generated C++ runs at a median of 0.98x the
> hand-written original. [Full results, including where we're slower ->]
> (/performance)

The line uses the median and the share within 1.1x, never a best case.
If the data is older than the current release it says so.

## 6. Real problems

Heading: "Real contest problems, rewritten". Three cards from the real
problems gallery (`70-project.md`), each with problem name, judge, tags,
line counts (C++ vs CP Lang) and a five-line excerpt. Picked by a
`featured: true` field in the corpus record, so the choice is data, not
code. [See all ->](/problems)

## 7. The toolchain

A strip of five items, each an icon, a name, one sentence and a link to
its tools page:

- **Stress testing**: `cpl stress` against a brute, shrinking the
  failing case.
- **Traces**: step through a run and see every value in source terms.
- **explain**: see the widths, layouts and costs the compiler chose.
- **Judge profiles**: the flags, stack and limits of each judge,
  locally.
- **Editor support**: diagnostics, hover and run lenses through `cpl
  lsp`.

## 8. Footer

Columns: *Learn* (course, quickstarts, tour), *Docs* (guides, reference,
library, errors, cheat sheet), *Tools* (CLI, setup, editors),
*Project* (status, design, decisions, performance, release notes,
contributing), *Community* (GitHub, discussions when they exist). A
version line: "Docs for CP Lang 0.3 · compiler 0.2.2 in the playground",
both read from the repo at build time.

## Meta

- Title: "CP Lang: a language for competitive programming".
- OpenGraph image: the hero's first tab rendered as an image.
- No analytics at launch.
