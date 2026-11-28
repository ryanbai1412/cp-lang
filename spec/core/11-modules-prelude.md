# 11. Files, modules, prelude and versions

Status: draft for 0.3.0. Sources: `plan/15-program-structure.md` §2, §12-14;
decisions 72, 94; `std/prelude.cp`.

## The model in one paragraph

A program file is statements; a library file is items and `use` lines,
with `pub` marking exports. Items hoist within their block, except a
`domain`, `type` or `const` whose value is read at runtime, which runs
where it is written. The prelude is exactly the vocabulary of the `lib/`
chapters' main tables (decision 150), so a contestant never writes a
`use` for an ordinary contest name.

## 11.1 Program files and library files

- A **program** file is a sequence of statements (chapter 10). It may
  declare items anywhere; items hoist within their block. A `domain`,
  `type` or `const` whose value is runtime (reads input or names a
  runtime value: `domain V = 1..=read<1..=2e5>()`, `type Z = Mod(p)`) is
  not hoisted: it runs where it is written, so input is read in source
  order, and a use above it is a use before definition. (R11.1.1)
- A **library** file contains only items (`fn`, `struct`, `enum`,
  `trait`, `impl`, `type`, `const`, `domain` with a constant count,
  `context`, `marker`) and `use` lines. `pub` marks exports; everything
  else is private to the file. (R11.1.2)
- An inline module `mod name:` groups items inside any file, for parts a
  contestant wants to stress alone or a grader entry (`@grader mod`). (R11.1.3)
- A struct's or enum's fields and methods are private to the module that
  declares them unless marked `pub`, the same rule as top-level names.
  Outside it, naming a non-`pub` member is E0207, whose fix adds `pub`;
  inside it every member is visible, so a single-file program never
  writes `pub`. A trait's methods have the trait's visibility, and std
  marks its API members `pub` (decision 358). (R11.1.4)

## 11.2 `use`

```cp
use std.graph.{dijkstra, Paths}
use std.segtree  // the stdlib: `segtree.SegTree`
use lib.flow as fl
use lib.treap  // the contestant's library root, from cp.toml
```

- Paths are module names, not files on a search path: `std` is the
  pinned stdlib, `lib` is the roots named in `cp.toml`. (R11.2.1)
- Libraries form an acyclic graph, and a program is a leaf of it: a
  library's modules may `use` each other, but two libraries, or a
  library and the program, that need each other are E0206. A library is
  what the snapshot reads it from (std's directory, or one `[libs]`
  entry), never its name. A `const` whose value reaches itself, through
  any modules, is still E0209. (R11.2.2)
- A local name shadows a prelude name, and so does a name a `use`
  imports, in method form too: after `use lib.mine.{step}`,
  `xs.step(2)` calls the library's `step` even where the prelude's also
  fits. A `use` that collides with a local name is an error naming
  both. (R11.2.3)
- Everything is inlined into the one output file and tree-shaken: only
  called functions and used generic instances cost bytes
  (`tools/bundle.md`). Library code is specialized where it is used, so
  it is as fast as code in the file. (R11.2.4)

## 11.3 The prelude

The prelude is the set of library names every program sees without a
`use`. It is one ordinary file, `std/prelude.cp`, made of `use` lines,
so what each name means is in its own module; the compiler has no
built-in list. Rules:

- Only programs get the prelude; library modules `use` what they need,
  so a library never depends on what a program happened to import. An
  inline `mod` inside a program is part of the program and sees its
  prelude (decision 241). (R11.3.1)
- The prelude is the vocabulary of the `lib/` chapters, by rule rather
  than by request (decision 150): every pub type and free function in a
  chapter's main tables is in it, together with every std type a prelude
  signature names (parameters, results, getters, yields), so any value a
  program can get it can also annotate; nothing else is.
  `std/prelude.cp` is the authoritative list; `scripts/spec-gate.py std-docs`
  checks it against the tables. Adding an API to a
  chapter's table adds it to the prelude; a helper the tables don't list
  needs a `use`. There is no name budget: the cost of a prelude name is a
  collision, not a count. (R11.3.2)
- Two exclusions keep that safe. A name a contestant commonly gives a
  local (`n`, `a`, `g`, `dp`, `ans`, `cnt`, `left`, `best`) is never a
  prelude name; the library spells it otherwise. And a name with two
  plausible meanings across chapters gets one chapter's meaning and the
  others are qualified (`min` and `max` stay the core fns over two or more
  values, ordinary declarations with a variadic `..more` past the first two
  (R06.1.5), as `abs`, `popcount`, `ctz`, `bit_width` and `bit_ceil` stay
  core builtins; the folds of one sequence are methods). (R11.3.3)
- A method call needs no import: `xs.sorted()`, `g.adjacent(v)` and
  `p.adj4` find the method on the receiver's type wherever the type
  came from. Only free names need the prelude or a `use`. (R11.3.4)
- `scripts/spec-gate.py std-docs` keeps `std/prelude.cp` and the chapter
  tables in step. The groups are: core types and traits; sequence
  vocabulary (`07-sequences.md`, `lib/sequence-patterns.md`); search and
  DP; graphs and trees; operations and range structures; containers;
  math and `Mod`; strings; I/O.
- The LSP offers an import quick fix for any non-prelude stdlib name,
  and `cpl fmt` sorts `use` lines. (R11.3.5)

## 11.4 Versions, pins and migration

- The first line of a solution is a compiler stamp, added by the
  formatter: `// cpl 0.3.0`, above any header comment. A file without one
  is valid and builds with the running compiler. The generated C++ carries
  the stamp and a source hash. (R11.4.1)
- `cp.toml` pins the stdlib and each library root by content hash.
  Old solutions rebuild byte for byte with their pinned compiler and
  libraries (`cpl build --compiler 0.3.0`). (R11.4.2)
- A breaking stdlib change ships a codemod: `cpl migrate` rewrites
  solutions and personal libraries, shows the diff, and records the new
  pin. Renames keep a deprecated alias for one minor version, with a
  diagnostic naming the new spelling. (R11.4.3)

## 11.5 What the submission contains

`cpl submit` produces one C++17 file with the program as `main` (or
the grader entry points), the library code it reaches, and the stamp.
It drops tests, brutes, generators, checkers, interactors, samples,
`debug(...)` calls and `@local` items.

## 11.6 Language roles

Some syntax needs a declaration to mean anything: `?T` needs a type and a
`for` needs a way to walk. Each such need is a **role**. The language fixes
the set of roles below. A library fills each role with an ordinary
declaration marked `@role(name)` where it is declared, and `std/prelude.cp`
exports every one of std's fillers. The compiler finds a role's declaration only through this
table. A declaration's name never gives it a role, so an `Optional`, `Show`
or `print` a program declares shadows that name and leaves the role alone.

| Role | std's filler | The syntax that uses it |
|---|---|---|
| `option` | `Optional<T>`, with `Some` and `None` | `?T`, `x?`, `a ?? b`, the patterns `Some(p)` and `None`, and `x == None` |
| `show` | `Show` | how `print`, `write` and `"{x}"` turn each value into text |
| `print` | `print` | `print(...)`: shows its values and ends the line, reading `io`'s `sep` and `precision` |
| `write` | `write` | `write(...)`: the same without the line end |
| `iteration` | `Seq`, with `iter` and `each` | `for x in s`, `s.each(f)` and `s.iter()` |
| `trace` | `traced`, `traced_bool` and `traced_option` | what a trace event (contract 8) shows of a value: the text the fn whose parameter takes the value writes, kept in the event instead of the output (Rtrace.7) |
| `todo` | `todo` | the expression `todo`, of any type, that traps when run (R01.6.7) |
| `recipe` | `Gen<T>` | the value of a `-> yields T` call made outside a `for` head, which a `for` over it runs (R07.6.2) |
| `positions` | `Domain` | the trait every domain's positions implement, and nothing else does |
| `order` | `Ordered` | stepping and order, which every core domain also implements (R04.2.31) |
| `convert` | `Into<T>` | `T(x)` for a core type `T`: `x`'s impl of the trait at `T` (R04.2.28) |
| `counted` | `Sized` | the trait whose one fn counts the positions of `domain V = xs` (R04.2.29) |
| `range_bounds` | `RangeEnds<D>` | the trait whose first and second fns give the cuts `[r]T` covers (R04b.1.5) |
| `endpoint` | `Endpoint` | the trait on a range's end type whose associated type `Range` is the struct `lo..hi` is: `Range<D>` for a domain `D`, `RealRange` for `F64` (decision 456) |
| `tuple` | `Tuple1<A>`, `Tuple2<A, B>` and the rest | the struct an n-tuple of its field count is, one filler per field count |
| `list` | `List<T>` | the struct `[a, b]` is, whatever a module names `List` |
| `str` | `Str` | the struct `"s"` is |
| `array` | `Array<D, T>` | the struct `[D]T` is |
| `map` | `Map<K, V>` | the struct `[k: v]` is |
| `cuts` | `Cut<D>` | the struct of a domain's cuts, `D.cuts` |
| `str_literal` | `str_of` | the fn that builds a string literal from its characters |
| `range_literal` | `range_of` | the fn that builds `lo..hi by k` |
| `interpolation` | `put_piece` | the fn each piece of an interpolated string calls, with its width, zero fill and digits (R10.3.2) |
| `derive_fields` | `Show` | the trait every struct without its own impl gets through its fields, packed as `pack` pairs (R08.1.6) |
| `memo_table` | `MemoTable` | the trait a `@memo_table(word)` struct's impl fills: `@memo(word)` reads with its `get` and keeps results with its `set` (R06.4.1) |
| `pack` | `Pack<H, T>` | the head-and-tail struct those fields and a variadic pack walk through |
| `variant_check` | `check_value` | the check an `Int` converted to an enum goes through |
| `prelude` | the module `std.prelude` | the names every program sees without a `use` (§11.3) |
| `record` | the module `std.tuple` | the home of every record shape's struct, row traits and `without` traits: a program's `{a: Int}` and `(Int, Str)` are its structs (R02.5.2, R08.4.1) |

The operator and literal markers of R06.4.12 (`@operator("op")`,
`@literal("[]")` and the rest) are the grammar's other roles. The
decorators the language itself defines, such as `@operator`,
`@on_unimplemented` and `@!multitest`, are part of this spec, not of any
library, and mean the same in every module.

Rules:

- The roles are exactly those above and in R06.4.12. A role is filled only
  by its marker: declaring a type, trait or fn with a filler's name gives
  it no role and changes no syntax. (R11.6.1)
- `@role(name)` may be written in any module, library or program. A second
  filler of one role in one program is E0202, naming both; `trace` takes
  one filler per value type, and two that both take one type are E0202,
  said when a traced run shows a value of that type. An unknown role
  name is E0201.
- A filler has the shape its role needs: `option` is a generic type with a
  one-part constructor `Some` and an empty one `None`, `show` a trait with
  one method, `iteration` a trait whose `iter` yields its elements,
  `positions`, `order`, `convert`, `counted` and `range_bounds` traits, `recipe` a generic
  struct with one fn-typed field, `print`,
  `write` and `todo` fns, and `prelude` and `record` modules (`@role(..)`
  above the module's first `use`). A filler
  of the wrong shape is E0606 at the marker.
- Syntax whose role has no filler in the program is E0201, naming the role
  and the marker that fills it.
