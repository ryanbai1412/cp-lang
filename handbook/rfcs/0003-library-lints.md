# RFC 0003: Libraries declare their own lints

| | |
|---|---|
| Owner | lint-fmt |
| Status | Accepted: spec accepted it, asking for an exact definition of a pure hole and one error per malformed declaration |
| Ryan to review | no |
| Changes | `spec/core/06-control-functions.md` (6.4 decorators), `spec/core/12-powers.md` (W12), `handbook/design/lint-fmt.md`, `handbook/guidelines/lints-checklist.md` |

## Summary

A library function can carry a lint that teaches it. The author writes the
hand-made code it replaces and the call that replaces it, as CP Lang code,
in an `@lint` decorator on the function. Any program that uses the library
then gets a warning with a one-step fix where it writes the long form. The
standard library uses the same decorator, so no lint about a std function
needs to be written in Rust.

## Problem

Today every lint lives in Rust, in `crates/cpl-lint`. A contestant who keeps
a personal library with `fn sq(x: Int) -> Int` cannot tell their own
future code "you wrote `d * d`, call `sq`", although the standard library's
lints say exactly that kind of thing about `sum` and `max`. That breaks two promises: user libraries get every power the standard
library has (`handbook/principles.md`, small core, no privileged stdlib),
and conveniences are instances of general mechanisms. It also keeps std
names inside the linter: 42 string literals in `crates/cpl-lint` name std
functions, and each std rename breaks a Rust rule.

## Proposal

```cp
/// The square of `x`.
@lint(square_by_hand, from: "x * x", to: "sq(x)")
pub fn sq(x: Int) -> Int:
    x * x
```

A program that does `use mylib` and writes `let d2 = (xa - xb) * (xa - xb)`
gets `warning[square_by_hand]: write sq(xa - xb)`, and `cpl fix`
rewrites it. `xs[i] * xs[j]` stays quiet: the two sides differ.

The rules:

1. `@lint(name, from: "<code>", to: "<code>")` decorates a `pub fn`.
   `from` and `to` are CP Lang code in string literals: an expression or one
   statement.
2. The decorated function's parameter names are the holes. In `from`, a
   hole matches any expression; a hole used twice must match the same
   expression twice. Every other name must match itself, except names the
   pattern binds itself (a lambda parameter, a loop variable), which match
   any name consistently.
3. `to` uses only holes and must call the decorated function. The fix
   substitutes the matched expressions, so it keeps the user's own spelling.
4. The finding is named by the lint (`warning[square_by_hand]`); it is in
   the `idiom` group, at `warn`, and `with lint(square_by_hand: allow)`
   turns it off like any other lint.
5. A hole written twice in `from` matches only an expression that calls
   nothing, so a fix never drops a repeated call.
6. The checker reports a malformed `@lint` where it is written (E0505):
   `from` or `to` that is not one expression or statement, a parameter
   `from` matches that `to` drops, a `to` that does not call the function,
   or a name declared twice in one module.

Matching is syntactic, on the parsed tree: one pass per file over the
patterns of the modules it uses, as fast as the existing rules.

## Cost

Spec: one bullet in 6.4 and the W12 status. Lint-fmt builds the matcher,
the checker validates the decorator (a few lines in the decorator check),
docs list library lints beside the built-in ones. If a library's patterns
make lint slow, the speed gate (`lints/R24`) fails on the corpus.

## Alternatives

- **Patterns with explicit holes (`$x`).** Needs new lexer syntax; the
  parameter names already say what the holes are.
- **Lints in a separate file per library (`lints.toml`).** Splits a
  function from the rule that teaches it, and is a second language to learn.
- **Rust plugins.** Excludes every contestant; the goal is no Rust.
- **Loop-form patterns (`from: fold`).** Stronger, but needs the loop-form
  recognizers first; a later RFC can add `from: <form>` beside code patterns.

## Prior art

Clippy and rustc lints are Rust only. Semgrep and Comby match code patterns
with holes in the target language. Haskell's HLint takes rules like
`warn: {lhs: map f (map g x), rhs: map (f . g) x}` from a library-side file,
with single-letter names as holes; this proposal uses the decorated
function's parameters instead.
