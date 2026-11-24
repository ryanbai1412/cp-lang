# Format use cases

### fmt/U1: format the file on save
Need: the file has one canonical layout without thinking about it.
Design: the editor's format-on-save calls the server, which writes what `cpl fmt` writes.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-lsp/src/tests/format.rs` (`formatting_replaces_the_file_with_what_cpl_fmt_writes`); `crates/cpl-lsp/tests/format_parity.rs` (`editor_formatting_equals_cpl_fmt_on_every_conformance_case`)

### fmt/U2: format a whole project from the shell
Need: every file in the project in the canonical layout, in one command.
Design: `cpl fmt .`; a directory stands for every `.cp` file under it.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-fmt/src/tests.rs` (`rewrites_files_under_a_directory`); integration `crates/cpl-cli/tests/golden.rs` (`fmt_check_reports_without_writing_and_fmt_rewrites`)

### fmt/U3: check without writing
Need: a hook or CI step fails when a file isn't formatted, and changes nothing.
Design: `cpl fmt --check` lists the files that would change and exits 1.
Milestone: M1-differentiating. Status: supported
Evidence: `crates/cpl-fmt/src/tests.rs` (`check_lists_files_and_changes_nothing`)

### fmt/U4: formatting is stable
Need: formatting twice gives the same file, and the layout never fights the user.
Design: `fmt(fmt(x)) == fmt(x)` on every file; a line's width counts the trailing comma formatting adds.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-fmt/src/tests.rs` (`formats_the_whole_corpus`, `counts_the_added_trailing_comma_against_the_width`); `scripts/fmt_check.py`

### fmt/U5: a small edit reformats only what it touched
Need: a one-line change makes a one-line diff.
Design: layout of a line depends on that line and its brackets, not on the rest of the file.
Milestone: M1-differentiating. Status: supported
Evidence: `crates/cpl-fmt/src/churn_tests.rs` (`a_one_line_spacing_edit_reformats_only_that_line`)

### fmt/U6: format a selection, or the line just typed
Need: tidy the part you are working on without touching the rest.
Design: range formatting widens to whole statements; on-type formatting tidies the line above a newline or a `:`.
Milestone: M1-differentiating. Status: supported
Evidence: `crates/cpl-fmt/src/range_tests.rs` (`formats_only_the_selected_statement`, `a_newline_formats_the_line_above_and_leaves_the_new_one`); integration `crates/cpl-lsp/src/tests/format.rs` (`typing_a_newline_formats_the_line_just_finished`)

### fmt/U7: format code that doesn't parse yet
Need: a half-written file still gets tidied, and nothing it holds is lost.
Design: formatting works on tokens and brackets; it never fuses two tokens, and a file the lexer can't read is refused with the error.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-fmt/src/tests.rs` (`formats_code_the_parser_rejects`, `never_fuses_two_tokens`, `refuses_a_file_the_lexer_cannot_read`)

### fmt/U8: choose indent and line width
Need: a project uses two spaces, or 80 columns.
Design: `[fmt]` in the nearest `cp.toml` with `indent_width` (2 to 8) and `line_width` (40 to 200); nothing else is configurable.
Milestone: M1-differentiating. Status: supported
Evidence: `crates/cpl-fmt/src/tests.rs` (`cp_toml_sets_the_indent_and_line_widths`); integration `crates/cpl-lsp/src/tests/format.rs` (`formatting_follows_the_fmt_section_of_cp_toml`)

### fmt/U9: keep comments where they are
Need: comments stay on their line and inside their brackets; trailing comments of consecutive lines line up.
Design: comments are tokens the layout keeps; a comment inside brackets keeps them open.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-fmt/src/tests.rs` (`keeps_comments_where_they_are`, `aligns_trailing_comments_of_consecutive_lines`, `a_comment_deep_in_brackets_keeps_them_open`)

### fmt/U10: sorted imports
Need: `use` lines in one order without sorting them by hand.
Design: each run of `use` lines is sorted, std first; a sort that would change a line's meaning is refused.
Milestone: later. Status: supported
Evidence: `crates/cpl-fmt/src/tests.rs` (`sorts_a_run_of_use_lines_std_first`, `sorting_that_changes_a_line_is_refused`)

### fmt/U11: a Windows checkout
Need: formatting doesn't rewrite every line end.
Design: CRLF line ends are kept, in whole-file and range formatting.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-fmt/src/tests.rs` (`keeps_crlf_line_endings_of_a_windows_checkout`); `crates/cpl-fmt/src/range_tests.rs` (`keeps_crlf_line_ends`)

### fmt/U12: an editor or script reads the result
Need: a tool knows which files changed and why one wasn't formatted.
Design: `cpl fmt --json`: `files`, each with `path`, `changed`, `ok`, `diagnostics` and `error`.
Milestone: later. Status: supported
Evidence: integration `crates/cpl-cli/tests/golden.rs` (`fmt_json_says_which_files_changed_and_why_one_was_not_formatted`)

### fmt/U13: docs show formatted code
Need: every snippet a reader copies is already in the canonical layout.
Design: a test formats every `cp` block in the docs and spec.
Milestone: M1-differentiating. Status: supported
Evidence: `crates/cpl-fmt/src/docs_tests.rs` (`every_doc_snippet_is_formatted`)

### fmt/U14: format in the playground
Need: the browser playground formats like the CLI.
Design: the wasm build runs the same formatter with the default style.
Milestone: M1-differentiating. Status: gap (no test pins playground output to `cpl fmt`)
Evidence: none yet
