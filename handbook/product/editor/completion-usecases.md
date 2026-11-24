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
Design: in `f(..` at the start of an argument, each parameter not given yet as `name:`, the named-only ones first; a struct's fields in its constructor. Typing `(` or `,` offers them, and only them.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lsp/src/tests/complete.rs` (`completion_offers_the_open_calls_argument_names_not_given_yet`, `a_typed_paren_comma_or_brace_completes_only_what_belongs_there`); integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases named/std-empty, named/user-fn, named/ctor, named/ctor-second

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
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases type/let, type/generic-arg, type/param, type/return, type/field, type/opt; `crates/cpl-lsp/src/tests/complete.rs` (`completion_ranks_types_above_values_where_a_type_goes`)

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
Design: the language server built to wasm behind the site's editor, answering every context of `crates/cpl-lsp/tests/completion-contexts.json` as `cpl lsp` does.
Milestone: M1-critical. Status: partial (the browser test types 2 of the contexts; the rest wait for a parity test over the shared cases)
Evidence: integration `web/tests/intelligence.mjs`

### complete/U19: in VS Code
Need: completion as soon as the extension is installed.
Design: the extension starts `cpl lsp`; `.`, `(`, `,` and `{` trigger completion, word characters trigger VS Code's own suggestions.
Milestone: M1-critical. Status: supported
Evidence: integration `editors/vscode/smoke.sh`; `editors/vscode/test/integration/suite.ts`

### complete/U20: in Neovim, Vim and JetBrains
Need: completion in the editor you already use.
Design: `cpl setup <editor>` configures the editor's LSP client for `cpl lsp`.
Milestone: M1-differentiating. Status: supported
Evidence: integration `editors/lsp-smoke.py`

### complete/U21: lints and levels in `with lint(..)`
Need: allow one lint on one block without looking up its exact name.
Design: in `with lint(`, every lint by name with its title; after `name:`, `allow`, `warn` and `deny`.
Milestone: M1-differentiating. Status: partial (a library's own `@lint`s are not offered yet)
Evidence: integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases lint/name, lint/level

### complete/U22: match arms on an optional or a struct
Need: write the arms of a `match` without remembering how the variants or fields are spelled.
Design: `Some` and `None` as the prelude names they are; a struct pattern's fields after `P(`.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases match/optional, match/field-pattern

### complete/U23: a member through `!` or `?`
Need: call a method of the value inside an optional, after asserting or propagating it.
Design: after `m!.` and `m?.`, the members of `T` in `?T`.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases dot/unwrap, dot/optional-chain

### complete/U24: inside a `@test` fn
Need: write a quick property check next to the solution.
Design: `assert` and the file's fns, as in any body.
Milestone: M1-differentiating. Status: supported
Evidence: integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases test/body

### complete/U25: the bounds of a domain and the type of a `read`
Need: declare an index space and its input from names already read.
Design: names in scope on the right of `domain V = 0..`; types in `read<..>`.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-lsp/tests/completion_contexts.rs` (`completion_offers_what_each_context_needs`), cases domain/rhs, type/read-arg

### complete/U26: snippets for contest shapes
Need: the skeleton of a multitest loop, a `@check` or a stress setup in one step.
Design: the VS Code extension ships contest snippets; the server offers none, so the other editors and the browser have none.
Milestone: M1-differentiating. Status: partial (VS Code only)
Evidence: integration `editors/vscode/smoke.sh`

### complete/U27: a browser example with a hidden setup prefix
Need: completion in a doc or Learn block that hides its setup lines.
Design: the prefix's names are offered and the cursor maps past the prefix.
Milestone: M1-critical. Status: gap
Evidence: none yet; site plans a browser parity test over the shared cases

### complete/U28: the practice judge editor and std reference examples
Need: the same completion wherever the site lets you edit code.
Design: every editable block runs the same wasm server.
Milestone: M1-differentiating. Status: gap
Evidence: none yet

### complete/U29: read-only examples offer nothing
Need: a static example never pops up a menu while you select text in it.
Design: completion only in editable blocks.
Milestone: M1-differentiating. Status: gap
Evidence: none yet

### complete/U30: before the compiler loads, or after it fails
Need: know completion isn't ready yet rather than think it is broken.
Design: until cpl.wasm loads, or after it fails, the editor says so and completion recovers when it loads.
Milestone: M1-critical. Status: gap
Evidence: none yet

### complete/U31: the keyboard in the browser editor
Need: the keys you use in any editor.
Design: Ctrl+Space opens the menu, Enter and Tab accept, Esc closes.
Milestone: M1-differentiating. Status: gap
Evidence: none yet

### complete/U32: the menu on a phone
Need: read and pick a completion on a narrow screen.
Design: the menu and its detail fit the viewport width.
Milestone: later. Status: gap
Evidence: none yet
