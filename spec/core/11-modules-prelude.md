# 11. Files, modules, prelude and versions

Status: draft for 0.3.0. Sources: `plan/15-program-structure.md` §2, §12-14;
decisions 72, 94; `spec/std/prelude.cp`.

## 11.1 Program files and library files

- A **program** file is a sequence of statements (chapter 10). It may
  declare items anywhere; items hoist within their block.
- A **library** file contains only items (`fn`, `struct`, `enum`,
  `trait`, `impl`, `type`, `const`, `domain` with a constant count,
  `context`, `marker`) and `use` lines. `pub` marks exports; everything
  else is private to the file.
- An inline module `mod name:` groups items inside any file, for parts a
  contestant wants to stress alone or a grader entry (`@grader mod`).

## 11.2 `use`

```
use std.segtree                  // the stdlib: `segtree.SegTree`
use std.graph.{dijkstra, Paths}
use lib.treap                    // the contestant's library root, from cp.toml
use lib.flow as fl
```

- Paths are module names, not files on a search path: `std` is the
  pinned stdlib, `lib` is the roots named in `cp.toml`.
- Modules form an acyclic graph; two modules that need each other are
  one module.
- A local name shadows a prelude name; a `use` that collides with a
  local name is an error naming both.
- Everything is inlined into the one output file and tree-shaken: only
  called functions and used generic instances cost bytes
  (`tools/bundle.md`). Library code is specialized where it is used, so
  it is as fast as code in the file.

## 11.3 The prelude

The prelude is the set of library names every program sees without a
`use`. It is one ordinary file, `std/prelude.cp`, made of `use` lines,
so what each name means is in its own module; the compiler has no
built-in list. Rules:

- Only programs get the prelude; library modules `use` what they need,
  so a library never depends on what a program happened to import.
- The prelude is the vocabulary of the `lib/` chapters, by rule rather
  than by request (decision 150): every pub type and free function in a
  chapter's main tables is in it, and nothing else is. Adding an API to a
  chapter's table adds it to the prelude; a helper the tables don't list
  needs a `use`. There is no name budget: the cost of a prelude name is a
  collision, not a count.
- Two exclusions keep that safe. A name a contestant commonly gives a
  local (`n`, `a`, `g`, `dp`, `ans`, `cnt`, `left`, `best`) is never a
  prelude name; the library spells it otherwise. And a name with two
  plausible meanings across chapters gets one chapter's meaning and the
  others are qualified (`min` and `max` stay the binary builtins; the
  folds are methods).
- `scripts/spec-stub.py --check` keeps `std/prelude.cp` and the chapter
  tables in step. The groups are: core types and traits; sequence
  vocabulary (`07-sequences.md`, `lib/sequence-patterns.md`); search and
  DP; graphs and trees; operations and range structures; containers;
  math and `Mod`; strings; I/O.
- The LSP offers an import quick fix for any non-prelude stdlib name,
  and `cpl fmt` sorts `use` lines.

## 11.4 Versions, pins and migration

- The first line of a solution is a compiler stamp, added by the
  formatter: `// cpl 0.3.0`. The generated C++ carries the stamp and a
  source hash.
- `cp.toml` pins the stdlib and each library root by content hash.
  Old solutions rebuild byte for byte with their pinned compiler and
  libraries (`cpl build --compiler 0.3.0`).
- A breaking stdlib change ships a codemod: `cpl migrate` rewrites
  solutions and personal libraries, shows the diff, and records the new
  pin. Renames keep a deprecated alias for one minor version, with a
  diagnostic naming the new spelling.

## 11.5 What the submission contains

`cpl submit` produces one C++17 file with the program as `main` (or
the grader entry points), the library code it reaches, and the stamp.
It drops tests, brutes, generators, checkers, interactors, samples,
`debug(...)` calls and `@local` items.
