# Completion use cases

### complete/U1: a member of a named value
Need: you hold a value and want what you can do with it.
Design: after `name.`, the type's fields and methods, then the std fns whose `self` takes it, each with its signature.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases dot/field, dot/list, dot/str, dot/map, dot/std-type; `crates/cpl-lsp/src/tests/complete.rs` (`completion_after_a_trailing_dot_lists_fields_getters_and_methods`)

### complete/U2: a member of a call's or an index's result
Need: chain the next operation without naming the intermediate value.
Design: after `f(..).`, `a[i].` and `xs.sorted().`, the members of what the expression returns.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases dot/after-call, dot/after-index, dot/chain; `crates/cpl-lsp/src/tests/complete.rs` (`dot_completion_after_a_method_call_lists_what_it_returns`)

### complete/U3: a member of a name with no written type
Need: members of a lambda parameter, a loop variable or `self`, whose type you never wrote.
Design: the type the checker inferred for it, at the cursor.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases dot/lambda-param, dot/loop-var, dot/self, dot/in-fn-param

### complete/U4: a member of an optional
Need: test or transform a `?T` without unwrapping it first.
Design: after `m.` on a `?T`, the std fns that take `self: ?T` (`is_some_and`, `and_then`, `is_none_or`).
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lsp/src/tests/complete.rs` (`completion_works_after_a_dot_in_an_optional_an_interpolation_and_a_use`)

### complete/U5: a member inside a string's interpolation
Need: format an answer line without writing the value on a line of its own first.
Design: inside `"{..}"`, completion works as in code; in the string's text, and after `{{`, it offers nothing.
Milestone: M1-differentiating. Status: supported
Evidence: `crates/cpl-lsp/src/tests/complete.rs` (`completion_works_after_a_dot_in_an_optional_an_interpolation_and_a_use`)

### complete/U6: a type's own members: variants, constructors, domain members
Need: build a value or name a variant without remembering how the type spells it.
Design: after `Dir.`, the variants; after `Graph.`, the fns without `self`; after `V.`, the domain's members; in a match arm, the variants.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases dot/enum, dot/static, dot/domain, match/variant

### complete/U7: an argument's name in an open call
Need: give an option like `by:` without remembering its spelling or its place.
Design: in `f(..` at the start of an argument, each parameter not given yet as `name:`, the named-only ones first; a struct's fields in its constructor.
Milestone: M1-critical. Status: partial (only `.` triggers completion, so the names come after a first letter, not on `(` or `,`)
Evidence: `crates/cpl-lsp/src/tests/complete.rs` (`completion_offers_the_open_calls_argument_names_not_given_yet`); integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases named/std-empty, named/user-fn, named/ctor, named/ctor-second

### complete/U8: a name in scope
Need: reuse a local, a parameter or a fn without retyping it in full.
Design: locals, parameters, loop and lambda variables, the file's declarations (also ones below the cursor), imports and the prelude, each with its signature.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lsp/src/tests/complete.rs` (`completion_offers_locals_declarations_and_the_prelude_with_signatures`); integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases name/local, name/param, name/lambda-param, name/later-fn

### complete/U9: a library fn by the start of its name
Need: call `runs(` or `SegTree(` when you remember only its start.
Design: prelude names complete in scope; a std name the file doesn't import yet comes with the `use` it needs.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lsp/src/tests/fixes.rs` (`completion_offers_unimported_stdlib_names_with_their_use`); integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases name/prefix-call, name/unimported

### complete/U10: a module or item in a `use`
Need: import what you need without knowing which module it is in.
Design: modules after `use std.`, with their summaries; a module's public items after `use std.graph.` and inside `{..}`.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases use/module, use/items, use/items-second, use/item-dot

### complete/U11: a type
Need: write an annotation, a parameter type, a return type or a generic argument.
Design: the types in scope (built-in, the file's, imported, std not imported yet).
Milestone: M1-critical. Status: partial (values are listed beside the types and ranked above them)
Evidence: integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases type/let, type/generic-arg, type/param, type/return, type/field, type/opt

### complete/U12: what fits comes first
Need: the right name on top when many are offered.
Design: entries whose type is the one expected (an argument, an annotated `let`, an assignment) rank first.
Milestone: M1-differentiating. Status: partial (the value after `by:` isn't ranked by the parameter's type)
Evidence: `crates/cpl-lsp/src/tests/complete.rs` (`completion_ranks_first_what_fits_the_expected_type`)

### complete/U13: completion while the file doesn't parse
Need: completion mid-edit, when a line above or below is unfinished.
Design: names come from what parses, members from the text before the cursor; a broken declaration still gives its names.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lsp/src/tests/complete.rs` (`completion_and_hover_see_names_from_a_declaration_that_did_not_parse`); integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases dot/broken-before, dot/incomplete-after, dot/eof-no-newline

### complete/U14: every identifier of a real solution
Need: whatever you were about to type is offered, not only the cases someone listed.
Design: the corpus ports retyped one identifier at a time; recall per context is a ratchet.
Milestone: M1-critical. Status: partial (recall is below 100%; the misses print as the backlog)
Evidence: integration `crates/cpl-lsp/tests/completion_replay.rs`

### complete/U15: keywords and decorators
Need: `while`, `match`, `@alt`, `@check` without typing them in full.
Design: keywords and the decorators in scope complete like names.
Milestone: M1-differentiating. Status: supported
Evidence: integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases name/keyword, name/annotation; `editors/vscode/test/integration/suite.ts`

### complete/U16: what an entry is before you pick it
Need: tell `sorted` from `sorted_by_key` without leaving the line.
Design: each entry shows its signature, its doc and the documented cost.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-lsp/tests/std_items.rs` (every public std item's completion detail, doc and cost)

### complete/U17: as fast as typing
Need: the list appears before you type the next letter, in a long file too.
Design: completion within the editor latency budget on a 5,000-line file.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-lsp/tests/latency.rs`

### complete/U18: in the browser editor
Need: the same completion in the playground and the guide's code blocks, with nothing installed.
Design: the language server built to wasm behind the site's editor.
Milestone: M1-critical. Status: supported
Evidence: integration `web/tests/intelligence.mjs`

### complete/U19: in VS Code
Need: completion as soon as the extension is installed.
Design: the extension starts `cpl lsp`; `.` triggers completion, word characters trigger VS Code's own suggestions.
Milestone: M1-critical. Status: partial (only `.` is a trigger character)
Evidence: integration `editors/vscode/smoke.sh`; `editors/vscode/test/integration/suite.ts`

### complete/U20: in Neovim, Vim and JetBrains
Need: completion in the editor you already use.
Design: `cpl setup <editor>` configures the editor's LSP client for `cpl lsp`.
Milestone: M1-differentiating. Status: supported
Evidence: integration `editors/lsp-smoke.py`
