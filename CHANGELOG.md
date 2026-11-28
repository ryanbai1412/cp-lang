# Changelog

One section per version: language, library, tools, and breaking changes
with how to migrate. The website's release notes are generated from this
file.

## 0.3.0 (unreleased)

The 0.3.0 language runs on the edge channel (`cpl --version` prints
`cpl 0.3.0-edge+<commit>`). handbook leads design; spec/ is its downstream
exhaustive form, and `plan/STATUS.md` lists what is still open.

### Language

- Enum variants are PascalCase: `Some`, `None`, `Anchor.Start`. In a
  pattern, a capitalised name is a variant and a lowercase name binds
  (decision 157).
- Memoized functions are `@memo(array)` or `@memo(map)`; `@cache` marks a
  struct field a read may write (decision 145).
- Decorators go before `pub`: `@get pub fn`.
- `-> lends T` always has the arrow.

### Library

- `lo..hi` over `F64` builds a `RealRange`, whose `bsearch`, `peak` and `valley` take `steps:`. On a
  domain range `bsearch` returns a `Cut<D>`, and the `Search` trait is gone (decision 672).
- `blocks(g)` walks one flat copy of the arcs with a single DFS path and edge stack instead of
  building a list of arcs per vertex: about 1.5x faster, and below hand-written C++ on
  cco16p3's largest input.
- `hash_token` comes from the `Hashable` trait: `Int`, `Char`, `Bool`, domain positions and
  `HashVal` implement it, so `HashVal.hash_token` reads the same.
- `adjacency(arcs)` takes any walk of arcs with `u` and `v` fields (more fields allowed) and stays
  directed: an arc lands only in its `u`'s list.
- A version of a persistent tree or array reads as a container through the
  store that made it: `v.within(t)`, through `trait Versioned` (decision 555).
- `Filtered.src`, `Mapped.src` and `Items.src` are now `pub` fields rather
  getter fns; `v.src` reads the same.
- The library is respecified in `std/`: about 1,700 documented
  signatures, each with its cost.
- Prefix sums and Fenwick trees answer range queries with `fold(r)`, not
  `sum`.
- Substring search is `s.search(p)` and `s.rsearch(p)`; `find` and `rfind`
  take predicates only.
- `collect()` and `sorted()` return a `List` keyed from 0.
- New general mechanisms found by porting 100 real problems: `buckets`,
  `read_all`, `pieces`, `closure` and `try_from_parents` (decisions 158 to
  165).

### Breaking changes

- `--json` keys follow one naming rule (`spec/tools/cli.md`): elapsed
  time is `time_ms` (`time_ns` in `cpl profile`), peak memory `peak_kb`,
  limits `time_limit_ms` and `memory_limit_kb` (`cpl judges` and `cpl bench`),
  and `cpl judges`' thread stack `thread_stack_kb`,
  the source `path`, a pass flag `ok`, and a count `<noun>_count`. Scripts
  that read `ms`, `kb`, `nanos`, `file`, `passed`, `total`, `samples`,
  `inputs`, `lines`, `covered`, `saved_passed`, `cpl stress`'s `cases` or
  `cpl judges`' `time_ms`, `memory_mb` and `thread_stack_mb` must read the
  new names.
- `none` becomes `None` and `some` becomes `Some`. The 0.2.2 compiler
  already accepts both spellings.
- `D.bounds` is now `D.cuts`, `(+, max)` is `each(+, max)`,
  `.enumerate()` is `.keyed().items`, `@alt(main)` is `@alt(program)`, and
  `@allow(x)` is `with lint(x: allow)`. Each old spelling is error E0503,
  whose fix writes the new one; `cpl migrate` applies them all.
- std's 2-D and list prefix sums are no longer `_Prefix2`, `_PrefixList`,
  `_FillPrefix2`, `_FillPrefixList` and `_Diff2`: std declares `Prefix<T>`,
  `Prefix<R, C, T>`, `FillPrefix<..>` and `Diff<R, C, T>` by those names
  (same-name structs that differ by arity), so `_Diff2.finish`,
  `_Prefix2.filling` and `_PrefixList.filling` are `Diff.finish` and
  `Prefix.filling`. A program that wrote the underscore names writes the
  public ones.

## 0.2.2 (2026-11-20)

The first numbered release of the language and its toolchain.

### Tools

- `cpl`: `check`, `fix`, `run` (the debug interpreter), `build` (one C++17
  file to submit), `fmt`, `lint`, `explain` and `doc`.
- Contest workflow: `new`, `test`, `watch`, `bench`, `stress` (against a
  brute, shrinking the failing case), `profile`, `cover` and `submit`.
- Judge profiles for generic C++17, Codeforces, AtCoder, CSES, ICPC and
  more, selected per file or project (`cpl judges`).
- `cpl lsp` for every editor, with support for VS Code, Neovim, Helix, Zed,
  Emacs and JetBrains IDEs in `editors/`; `cpl dap` for time-travel
  debugging.
- A WASM build of the compiler that runs in the browser.
