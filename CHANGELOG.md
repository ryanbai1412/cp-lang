# Changelog

One section per version: language, library, tools, and breaking changes
with how to migrate. The website's release notes are generated from this
file.

## 0.3.0 (unreleased)

The 0.3.0 language runs on the edge channel (`cpl --version` prints
`cpl 0.3.0-edge+<commit>`). Its normative source is `spec/`; `spec/STATUS.md`
lists what is still open.

### Language

- Enum variants are PascalCase: `Some`, `None`, `Anchor.Start`. In a
  pattern, a capitalised name is a variant and a lowercase name binds
  (decision 157).
- Memoized functions are `@memo(array)` or `@memo(map)`; `@cache` marks a
  struct field a read may write (decision 145).
- Decorators go before `pub`: `@get pub fn`.
- `-> lends T` always has the arrow.

### Library

- The library is respecified in `spec/std/`: about 1,700 documented
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

- `none` becomes `None` and `some` becomes `Some`. The 0.2.2 compiler
  already accepts both spellings.
- `D.bounds` is now `D.cuts`, `(+, max)` is `each(+, max)`,
  `.enumerate()` is `.keyed().items`, `@alt(main)` is `@alt(program)`, and
  `@allow(x)` is `with lint(x: allow)`. Each old spelling is error E0503,
  whose fix writes the new one; `cpl migrate` applies them all.

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
