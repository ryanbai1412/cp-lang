# Fix use cases

### fix/U1: apply every fix an error carries
Need: errors that name their fix are fixed in one command.
Design: `cpl fix a.cp` applies each error's fix, re-checks, and rewrites the file.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-cli/tests/golden.rs` (`fix_applies_every_fix_and_reports_what_is_left`); integration `crates/cpl-cli/tests/catalogue.rs` (`every_emitted_code_has_an_exact_trigger_and_a_clean_fix`)

### fix/U2: then the safe lint fixes
Need: once the file checks, the lint fixes too, without a second command.
Design: when no error is left, `cpl fix` applies what `cpl lint --fix` applies.
Milestone: M1-differentiating. Status: supported
Evidence: integration `crates/cpl-lsp/src/tests/fixes.rs` (`fix_all_rewrites_removed_syntax_then_the_lints_it_uncovers`)

### fix/U3: a fix never changes what the program does
Need: trust the rewrite without reading it.
Design: `cpl fix` over every conformance case and port input keeps its output.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-cli/tests/fix_behavior.rs` (`fixes_keep_behaviour_0`)

### fix/U4: a fix never adds an error
Need: running `cpl fix` never leaves the file worse.
Design: every lint fix is re-checked over the corpus; a file it makes worse is a listed, shrink-only exception.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lint/src/corpus_tests.rs` (`fixes_never_add_checker_errors`); `crates/cpl-lint/src/record_tests.rs` (`every_record_fix_adds_no_checker_error`)

### fix/U5: see what is left
Need: know which errors still need a human, and fail a script while any remain.
Design: `cpl fix` prints the errors left in the rewritten file and exits 1 while one is left.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-cli/tests/golden.rs` (`fix_applies_every_fix_and_reports_what_is_left`)

### fix/U6: overlapping fixes
Need: two fixes on the same text don't corrupt it.
Design: the first applies; the second waits for the next round over the rewritten file.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lint/src/tests.rs` (`overlapping_fixes_wait_for_the_next_round`)

### fix/U7: the same fixes in the editor
Need: a quick fix in the editor does what `cpl fix` does.
Design: the server's code actions are the fixes the diagnostics carry.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-lsp/src/tests/fixes.rs` (`quick_fixes_come_from_the_diagnostics`, `the_preferred_quick_fix_is_the_one_cpl_fix_applies`, `a_library_lint_is_published_with_its_quick_fix`)

### fix/U8: move to a renamed spelling
Need: code using a removed spelling is moved to the new one.
Design: the error for a removed spelling carries the rewrite; `cpl fix` applies it.
Milestone: M1-differentiating. Status: supported
Evidence: `crates/cpl-lint/src/record_tests.rs` (`every_record_before_passes_cpl_check_or_names_its_removed_spelling`)

### fix/U9: a tool reads the result
Need: an editor or script knows how many fixes applied and what is left.
Design: `cpl fix --json`: `files`, the number `applied`, `ok`, and the `diagnostics` left.
Milestone: later. Status: supported
Evidence: integration `crates/cpl-cli/tests/golden.rs` (`fix_json_reports_what_it_applied_and_what_is_left`)

### fix/U10: a fix that needs a choice
Need: a fix that could be wrong is never applied unread.
Design: an error with several candidates offers each in the editor and none in `cpl fix`.
Milestone: M1-differentiating. Status: gap (no test pins that `cpl fix` skips a multi-candidate fix)
Evidence: none yet
