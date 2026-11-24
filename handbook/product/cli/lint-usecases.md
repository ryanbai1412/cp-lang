# Lint use cases

### lint/U1: legal but probably wrong code is pointed out
Need: see a likely bug the checker accepts, with the reason and the fix.
Design: a lint with a code, a one-line reason, a help naming the exact replacement and, where safe, a fix.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lint/src/record_tests.rs` (`every_lint_record_fires_on_before_only`, `every_lint_has_a_fix_or_says_why_not`)

### lint/U2: the same findings in the editor and the shell
Need: what the editor underlines is what `cpl lint` prints.
Design: one analysis feeds both; quick fixes are the lint's fixes.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-cli/tests/lsp_parity.rs` (`the_editor_reports_what_check_and_lint_report_on_every_v03_file_0`); `crates/cpl-lint/src/corpus_tests.rs` (`the_cli_and_the_editor_read_the_same_facts`)

### lint/U3: apply every safe fix at once
Need: accept the suggestions without editing by hand.
Design: `cpl lint --fix` applies the safe fixes, rounds until none overlap, then reports what is left.
Milestone: M1-differentiating. Status: supported
Evidence: `crates/cpl-lint/src/tests.rs` (`fix_rewrites_the_file_and_reports_what_is_left`, `overlapping_fixes_wait_for_the_next_round`)

### lint/U4: turn one lint off where it's wrong
Need: silence a finding for one block without touching the rest.
Design: `with lint(name: allow)` covers what follows it and nothing else.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lint/src/tests.rs` (`a_lint_context_covers_what_follows_it_and_nothing_else`); `crates/cpl-lint/src/checker_tests.rs` (`with_lint_allows_a_checker_lint_by_name`)

### lint/U5: set levels for a project
Need: a project allows, warns on or denies a lint or a whole group.
Design: `[lints]` in `cp.toml`: `name = "deny"`, `idiom = "allow"`; mistakes are reported by line with the closest name.
Milestone: M1-differentiating. Status: supported
Evidence: `crates/cpl-lint/src/tests.rs` (`cp_toml_sets_levels`, `cp_toml_sets_a_level_for_a_whole_group`, `cp_toml_mistakes_are_reported_by_line`)

### lint/U6: warnings don't block a run
Need: a warning never stops `cpl run` or `cpl lint` from succeeding; a deny does.
Design: exit 0 on warnings alone, 1 on a denied lint.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lint/src/tests.rs` (`warnings_do_not_fail_and_deny_does`); integration `crates/cpl-cli/tests/golden.rs` (`lint_warns_with_its_code_and_exits_0_on_warnings_alone`)

### lint/U7: a file that doesn't check is not called clean
Need: never read "0 warnings" on a file with errors.
Design: `cpl lint` reports the file as failing `cpl check`.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lint/src/tests.rs` (`a_file_that_does_not_parse_is_an_error_not_a_pass`); `crates/cpl-lint/src/checker_tests.rs` (`a_file_that_does_not_check_is_not_reported_clean`)

### lint/U8: understand a lint
Need: why it fires, what to write instead, and how to turn it off.
Design: `cpl explain L0536` or `cpl explain sentinel_scalar`: the rule, a before and after, and the level knobs.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lint/src/checker_tests.rs` (`every_checker_lint_has_a_registry_entry_for_cpl_explain`); `crates/cpl-lint/src/record_tests.rs` (`every_lint_matches_the_spec_lint_list`)

### lint/U9: a library's own lint
Need: a personal library warns about misuse of its API like the stdlib does.
Design: `@lint(name, from:, to:)` on a library fn; it fires, fixes, takes levels and is explained like a built-in.
Milestone: M1-differentiating. Status: supported
Evidence: `crates/cpl-lint/src/library_tests.rs` (`a_library_lint_fires_and_fixes_in_a_program_using_the_library`, `cp_toml_sets_a_library_lints_level`); integration `crates/cpl-cli/tests/cli.rs` (`explain_names_a_library_lint`)

### lint/U10: keep the input a bound lint found
Need: the input that breaks a proven bound becomes a test.
Design: `cpl lint --save` writes it to `d.tests/`, which `cpl test` runs.
Milestone: M1-differentiating. Status: supported
Evidence: integration `crates/cpl-cli/tests/lint_witnesses.rs` (`every_bound_lint_witness_traps_where_the_lint_says`)

### lint/U11: review a port for C++ habits
Need: list the translated idioms in a finished port, file by file.
Design: `cpl lint --taste`: a line per file, then totals by lint.
Milestone: later. Status: supported
Evidence: `crates/cpl-lint/src/tests.rs` (`taste_reports_a_line_per_file_and_the_total`)

### lint/U12: one finding per root cause
Need: a nested mistake shows once, not as a cascade.
Design: overlapping findings are merged or listed as distinct causes; a lint doesn't repeat the checker's error at its place.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lint/src/corpus_tests.rs` (`nested_findings_shown_together_teach_two_things`); `crates/cpl-lint/src/checker_tests.rs` (`a_lint_does_not_repeat_the_error_at_its_place`)

### lint/U13: a lint stays quiet on the right idiom
Need: no finding on code that is already the best expression.
Design: every lint has a `near_miss.cp` it must not fire on, and corpus and std warning counts only go down.
Milestone: M1-critical. Status: partial (ports still show false positives; each found becomes a quiet case below)
Evidence: `crates/cpl-lint/src/corpus_tests.rs` (`corpus_warnings_only_go_down`); `crates/cpl-lint/src/record_tests.rs` (`every_lint_record_fires_on_before_only`)

### lint/U14: stay quiet when the suggested call wouldn't run
Need: a help never suggests code that fails `cpl check`.
Design: a lint is quiet when its replacement doesn't check (`(m..n by k).collect()`), or suggests a form that does (a typed lambda when a fold's fn takes a wider type).
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lint/src/record_tests.rs` (`every_record_fix_adds_no_checker_error`); `crates/cpl-lint/src/corpus_tests.rs` (`fixes_never_add_checker_errors`); `crates/cpl-lint/src/idiom_tests.rs` (`loop_reduction_keeps_a_widening_call_in_a_typed_lambda`)

### lint/U15: stay quiet when a structure is kept for what the stdlib form loses
Need: no "use the stdlib type" finding when the hand-rolled one serves something the replacement can't: an ordered query on a map, a queue head that is also a rank, buckets keyed by a computed value, a per-vertex child list.
Design: the lint checks how the value is used, not only how it is built; any use outside the replacement's API keeps it quiet.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lint/src/tests.rs` (`domain_keyed_set_is_left_alone_when_sparse_or_ordered`); `crates/cpl-lint/src/corpus_tests.rs` (`corpus_warnings_only_go_down`)

### lint/U16: stay quiet when the shape only looks like the idiom
Need: no finding on a DP case split that resembles a grid neighbour read, a triple loop that resembles `pairs()`, or `D(x.ord + k)` where `D` starts at 0.
Design: the lint matches the whole shape (2-D, both indices guarded, no `else`; loop bounds free of enclosing variables; the domain's first label) before firing.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-lint/src/tests.rs` (`position_arithmetic_points_at_next_prev_and_offset`); `crates/cpl-lint/src/corpus_tests.rs` (`corpus_warnings_only_go_down`)

### lint/U17: a lint reads types the way the checker does
Need: a lint never misfires because a type was spelled through an alias, inferred, or written differently.
Design: lints read types, names and proven ranges from checker facts, not from written syntax.
Milestone: M1-differentiating. Status: partial (27 sites still read written types; `lint_type_syntax` in `scripts/debt-ratchet.json` only goes down)
Evidence: `crates/cpl-lint/src/idiom_tests.rs` (`sentinel_table_reads_the_element_type_from_the_checker`, `manual_adjacency_list_reads_the_list_type_from_the_checker`); `scripts/debt-ratchet.py`

### lint/U18: lint is fast enough to run on every keystroke
Need: findings appear while typing, even in a 4000-line library.
Design: lint and format of the largest files fit a tenth of the 500 ms front-end budget.
Milestone: M1-critical. Status: partial (the largest std modules lint in 30 to 40 ms in a release build, printed but not asserted; the ports are asserted under 50 ms)
Evidence: `crates/cpl-lint/src/speed_tests.rs` (`lint_and_fmt_of_the_largest_files_fit_a_tenth_of_the_front_end_budget`)

### lint/U19: lint in the playground
Need: the browser playground shows the same lints as the editor.
Design: the wasm build runs the same lint pass over the same analysis.
Milestone: M1-differentiating. Status: gap (no test pins playground findings to `cpl lint`)
Evidence: none yet

### lint/U20: tools read findings as JSON
Need: an editor or script consumes findings.
Design: `cpl lint --json` prints what `cpl check --json` prints.
Milestone: later. Status: supported
Evidence: integration `crates/cpl-cli/tests/golden.rs` (`lint_json_prints_the_files_cpl_check_json_prints`)
