# cpl-lint

`cpl lint` reports code that is legal but probably wrong, slow or
unidiomatic (`plan/45-dev-tools.md`). A lint never blocks a build. Each
has a stable code, a default level, a message that says what to write
instead, and a fix where one is safe.

```
$ cpl lint d.cp
warning[L0412]: `Map<V, Int>` is keyed by the domain `V`
  --> d.cp:3:10
  |
3 | var cnt: Map<V, Int> = Map<V, Int>()
  |          ^^^^^^^^^^^
  = help: `[V]?Int` uses one slot per `V` and has no lookup cost
```

`cpl lint --fix` applies the safe fixes and reports what is left;
`cpl lint --list` prints every lint. Warnings exit 0, so the command can
sit in a save hook; only a lint set to `deny` exits 1.

## The lints

- `position_arithmetic` (L0101): `V(v.ord + 1)` builds a position from a
  number. `.ord` counts from 0 and `V(..)` takes a label, so the two differ
  unless `V` starts at 0. The help names `v.next`, `v.prev` or
  `v.offset(k)`.
- `var_never_reassigned` (L0201): a `var` nothing assigns to, passes as
  `mut`, or calls a method on. The fix changes it to `let`.
- `unread_field` (L0202): a struct field that no `.name` reads. A file
  with no statements of its own is a library and is skipped.
- `shadows_top_level` (L0203): a parameter, local or loop variable with
  the name of a top-level `let`.
- `float_equality` (L0301): `==` or `!=` where either side is a float.
- `domain_keyed_map` (L0412): `Map<V, T>` or `Set<V>` where `V` is a
  domain; `[V]?T` and `[V]Bool` are the arrays to use.
- `memo_key_space` (L0413): a `@memo` keyed by domains whose table cannot
  fit in memory at the declared input bounds. With
  `let n: 1..=2e5 = read()` and `domain V = 1..=n`, `@memo fn go(a: V, b: V)`
  is 4.0e10 entries. `@memo(hash)` and number-typed keys are not tables
  and are left alone.
- `unproven_list_index` (L0414): `xs[i]` on a `List` inside two or more
  nested loops, where `i` is not the variable of `for i in 0..xs.len` or
  `for i in xs.indices`.
- `unknown_lint` (L0001): `with lint(...)` names a lint that does not exist.

## Levels

Each lint runs at `allow`, `warn` or `deny`. The nearest `cp.toml` at or
above the file sets them, and `with lint(name: allow)` turns one off for the
rest of its block, or for the block under it:

```toml
[lints]
float_equality = "deny"
shadows_top_level = "allow"
memory_mb = 64            # what memo_key_space checks against; 256 by default
```

```
with lint(unread_field: allow)
struct Event:
    time: Int
    kind: Int             // only read when events are sorted
```

## How it works

`lint(src, &config)` parses the file with `cpl-syntax` and runs each rule
over the tree; there is no type information yet, so a rule that needs a
type reads it off the source (an annotation, a float literal, a `List`
initializer) and stays quiet when it cannot tell. A file with a syntax
error gets no lints, because the part the parser could not read may hold
the assignment or the read a lint would say is missing.

- `src/lib.rs`: the lint table, `lint`, `with lint`, `apply_fixes`.
- `src/config.rs`: `[lints]` in `cp.toml`.
- `src/walk.rs`: the tree walk the rules share; it reports scopes and
  bindings, so a rule that follows names keeps a stack and nothing else.
- `src/rules/`: one file per rule or pair of related rules; `bounds.rs`
  turns declared input ranges into the sizes `memo_key_space` multiplies.
- `src/cli.rs`: the `cpl lint` command.

```
$ cargo test -p cpl-lint
```
