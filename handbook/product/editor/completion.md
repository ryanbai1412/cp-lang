# Completion

> 25 minutes in. You have the edges in `es: List<Edge>`, and the next line
> needs them sorted by weight. You type `es.so`, pick `sorted`, and type `(`.

`by:` is offered first, with its type, `by: T -> K`. You accept it and
write `e => e.w`. On the next line you want the heaviest edge's target:
`es.max(by: e => e.w).` offers `to` and `w`, because completion knows what
the call returns. Later you write `let best: ?Edge = None` and `best.`,
and `is_some_and` and `and_then` appear. In the output line `"{best.`
works too: inside a string's `{..}` you are writing code again. At no
point did you have to remember a name, look one up, or finish the line
first so the file would parse.

**The contexts.** Completion is driven by where the cursor is, and every
context has to answer:

- after `.`: fields, methods and the std fns that take the value as `self`,
  for any receiver: a name, a call's or an index's result, a lambda
  parameter, a loop variable, `self`, an optional, a tuple, a domain, a type
  (`Graph.`, `Dir.`);
- an argument's name in an open call: `by:` for std, a user fn's
  defaulted parameters, a struct's fields in its constructor, leaving out
  the ones already given;
- a name in scope: locals, parameters, fns declared later, the prelude,
  and std names not imported yet, which bring their `use`;
- a `use`: modules after `use std.`, items after `std.graph.` and inside
  `{..}`;
- a type: annotations, parameters, returns, generic arguments and fields;
- the rest: decorators after `@`, lints and levels in `with lint(`,
  match arms, `m!.` and `m?.`, a domain's bounds.

**Off the happy path.** The file almost never parses while you type, so
completion reads what does parse and the text around the cursor. A
declaration broken three lines up, or an unfinished `let z =` below,
changes nothing. In a comment or in a string's text it offers nothing.

**Ideal against today.** The ideal is that every identifier you were about
to type is in the list, ranked by what fits. The contexts above all
answer today (`crates/cpl-lsp/tests/completion_contexts.rs`), and over
the corpus, recall is a ratchet below 100%
(`crates/cpl-lsp/tests/completion_replay.rs`). The gaps, each a use case:

- In a type position, values are listed beside the types and ranked
  above them.
- The value after `by:` isn't ranked by the parameter's type.
- The browser editor is checked on 2 contexts, not all of them, and
  its loading and keyboard states are unchecked.
- `with lint(` doesn't offer a library's own `@lint`s.

**Against the usual setup.** A C++ contestant has clangd, which completes
members once the file compiles but offers nothing for a lambda's `auto`
parameter or an unfinished template call. Most of them type from memory.
Python's Pylance completes keyword arguments, the closest equivalent of
`by:`, but not members of a value whose type it can't infer. CP Lang knows
every type, because the checker inferred it, so completion can answer in
the places those tools can't.
