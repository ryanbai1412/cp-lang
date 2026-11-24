# Stress use cases

### stress/U1: compare the solution with a slow, right one in the same file
Need: an input where the solution and an obviously correct version disagree.
Design: `@alt(program) fn brute()` in the file; `cpl stress d.cp`.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-cli/tests/wrong_answer.rs`; `crates/cpl-cli/tests/seeded_bugs.rs` (every seeded bug found)

### stress/U2: compare one function with its slow version
Need: the solution is right except possibly one part, and only that part has a brute.
Design: `@alt(f) fn f_slow(..)`; stress runs the program with the slow part swapped in and names the first call that answers differently.
Milestone: M1-differentiating. Status: supported
Evidence: integration `crates/cpl-cli/tests/debugger.rs` (`stress_explain_names_the_first_call_of_a_part_its_alt_answers_differently`)

### stress/U3: compare with a brute written in another language
Need: the brute already exists as C++ or Python, or as last contest's solution.
Design: `--brute brute.cpp`, `.py`, `.cp` or any executable.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-stress/src/tests.rs` (`programs_resolve_by_extension`)

### stress/U4: find a crash or a time-out with no brute
Need: WA or RE on the judge, no brute written yet; find any input that breaks the run.
Design: `cpl stress d.cp` with nothing to compare: a case fails on a runtime error or the time limit, and the header says answers aren't checked.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-cli/tests/wrong_answer.rs`; trial: the first blind contest simulation hit "there is nothing to compare" here

### stress/U5: many answers are right
Need: judge an output that isn't the only correct one.
Design: a `@check fn` in the file reads the input and the output; `--checker prog` for an outside one.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-cli/tests/debugger.rs` (`the_loop_reruns_a_checked_multitest_through_the_files_checker`)

### stress/U6: an interactive problem
Need: stress a solution that talks to a judge.
Design: the file's `@interactor` plays the judge on each generated hidden test; `--interactor prog` for an outside one.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-cli/tests/debugger.rs` (`the_loop_reruns_an_interactive_multitest_through_the_files_interactor`)

### stress/U7: inputs with no generator written
Need: valid inputs without writing a generator.
Design: cases come from the `read()` types: ranges, domains, arrays, grids, permutations, trees, graphs, enums by their tags, words over an alphabet.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-cli/tests/stress_inputs.rs` (every generated case is accepted by the program's own reads); `crates/cpl-stress/src/tests/spec.rs`

### stress/U8: an input shape stress can't make
Need: know why no cases come, and what to write instead.
Design: the error names the `read()` and its type and suggests `@gen fn`; the shapes not made yet are listed.
Milestone: M1-critical. Status: partial (30 corpus programs read a shape stress can't make)
Evidence: `crates/cpl-stress/tests/readable_shapes.rs`, list `crates/cpl-stress/tests/generate-gaps.txt`

### stress/U9: a custom distribution
Need: cases the types don't describe (a guaranteed answer exists, a special structure).
Design: a `@gen fn` in the file, its choices shrunk like typed ones; `--gen prog` for an outside one, not shrunk.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-stress/src/tests.rs` (`a_generator_program_can_make_the_inputs`); `crates/cpl-cli/tests/stress_corpus.rs`

### stress/U10: a counterexample small enough to check by hand
Need: the smallest input that still fails.
Design: every failure is shrunk: numbers lowered, elements and test cases removed, trees and graphs kept valid.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-cli/tests/seeded_bugs.rs` (each bug shrinks to its `.min`); `crates/cpl-stress/src/tests.rs` (`shrinking_finds_the_smallest_case_that_still_fails`)

### stress/U11: read the case in the program's names
Need: see which value is which without counting tokens.
Design: `that is: n = 2  a = [3, 3]` under the raw input.
Milestone: M1-differentiating. Status: supported
Evidence: `crates/cpl-stress/src/tests.rs` (`a_case_renders_one_row_per_line_and_describes_itself_by_name`)

### stress/U12: which line is wrong
Need: go from a failing case to the line to fix.
Design: the report names the first value the solution and the brute give differently and the lines that made it; a runtime error shows its line and the locals.
Milestone: M1-differentiating. Status: partial (`--explain` is a flag, not part of every failure)
Evidence: `crates/cpl-cli/tests/seeded_bugs.rs` (`explain_names_the_line_each_bug_was_planted_on`, 29 of 29); `crates/cpl-cli/tests/wrong_answer.rs` (`explain_names_the_loop_that_never_ran`)

### stress/U13: step through the failing case
Need: see what the program did on the case stress found.
Design: the report's next step is `cpl run d.cp --trace`, which records the case stress saved last; `--trace` saves both traces side by side.
Milestone: M1-differentiating. Status: supported
Evidence: integration `crates/cpl-cli/tests/wrong_answer.rs`; `crates/cpl-cli/tests/debugger.rs` (`stress_finds_the_planted_bug_with_the_files_own_brute_and_traces_both`)

### stress/U14: keep the failing case as a test
Need: the case stays and proves the fix.
Design: saved as `d.tests/stress-<seed>.in` with the brute's output as `.out`; `cpl test d` reruns it.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-cli/tests/wrong_answer.rs`

### stress/U15: after the fix, back to a clean run in one command
Need: confirm the saved failures pass, then keep hunting.
Design: `cpl stress d` runs every saved `d.tests/stress-<seed>.in` before new cases. One that still fails is reported as it is, unshrunk and not saved again; the clean report says `the saved case passes`.
Milestone: M1-differentiating. Status: supported
Evidence: integration `crates/cpl-cli/tests/wrong_answer.rs` (`a_saved_case_runs_first_and_fails_until_it_is_fixed`, `a_wrong_answer_goes_from_a_crash_and_a_counterexample_to_a_clean_stress`)

### stress/U16: multitest input
Need: a failing multitest case cut to the one test that fails, or the two that leak state into each other.
Design: the `t` header is kept right while tests are removed.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-stress/src/tests.rs` (`a_multitest_input_has_a_header_and_shrinks_to_one_failing_test`, `a_bug_that_needs_two_tests_shrinks_both_of_them`)

### stress/U17: the solution hangs or is too slow
Need: an input that makes it run past the limit, and why.
Design: `--time-limit`; the case fails as TLE; `--explain` names the loop whose state repeats.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-cli/tests/seeded_bugs.rs` (`cold-lower-bound-stuck`)

### stress/U18: the brute is the one that fails
Need: not be told the solution is wrong when the brute crashed.
Design: the case fails as `brute RE`, shrunk like any other, and the brute's trap is shown as `cpl run` shows one: code, line, locals, help. It isn't saved as a test of the solution.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-cli/tests/wrong_answer.rs` (`a_brute_that_traps_is_blamed_with_its_line_and_locals_and_not_saved`, `a_swapped_in_brute_fn_that_traps_is_blamed_with_its_line_and_locals`); `crates/cpl-stress/src/tests.rs` (`a_failing_brute_is_a_failing_case_blamed_on_the_brute`)

### stress/U19: never blame the solution for an input it rightly rejects
Need: every case is one the problem allows.
Design: cases follow the declared ranges; a case the program's own reader rejects is a stress bug.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-cli/tests/stress_inputs.rs`

### stress/U20: the same case again
Need: reproduce a failure exactly, on another machine or after a restart.
Design: `--seed`; the same seed gives the same case and shrink for any `--jobs`.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-stress/src/tests.rs` (`stress_stops_at_the_first_failure_shrinks_it_and_is_the_same_for_any_jobs`)

### stress/U21: real-number answers
Need: outputs that differ only by rounding count as equal.
Design: tokens that are numbers match within 1e-6, absolute or relative, as judges do.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-stress/src/tests.rs` (`outputs_are_compared_by_token`)

### stress/U22: generated cases reach the edges
Need: `n = 1`, all equal, the maximum value and sorted input come up without asking.
Design: small sizes first; boundary values drawn more often than uniform.
Milestone: M1-differentiating. Status: partial (trees include paths and stars; nothing measures the rest)
Evidence: `crates/cpl-stress/src/tests.rs` (`random_trees_include_paths_and_stars`)

### stress/U23: stress from the editor
Need: run stress and read the case without leaving the editor.
Design: a Stress lens and command; `--format json` for the editor to show the case, the explanation, and a runtime error as the same diagnostic `cpl check --format json` gives.
Milestone: M1-differentiating. Status: partial (the JSON is tested; the lens test is TypeScript the gate can't run)
Evidence: integration `crates/cpl-cli/tests/wrong_answer.rs` (`a_runtime_error_in_the_json_report_is_a_diagnostic`); `crates/cpl-stress/tests/in_process.rs` (`a_runtime_error_in_the_in_process_report_is_a_diagnostic`); `editors/vscode/test/integration/suite.ts` (Stress lens saves a case)

### stress/U24: stress in the browser
Need: stress a solution without installing CP Lang.
Design: the playground's Stress tab, the same engine in the page.
Milestone: M1-differentiating. Status: partial (the engine matches the CLI; the browser tab isn't tested, and a time-out isn't explained in the browser)
Evidence: `crates/cpl-wasm/tests/parity.rs` (`stress_matches_cpl_stress_json`); `play/U8` in `handbook/product/site/playground-usecases.md`

### stress/U25: no C++ compiler on the machine
Need: stress still works on a fresh machine.
Design: without `g++`, both sides run in the interpreter.
Milestone: M1-critical. Status: supported
Evidence: integration `crates/cpl-cli/tests/debugger.rs` (`with_no_cpp_compiler_the_loop_runs_in_the_interpreter`)

### stress/U26: a cached field must not change the answer
Need: a `@cache` field gives the same answer cold.
Design: stress reruns with caches cold and compares, with no brute needed.
Milestone: later. Status: supported
Evidence: conformance `spec/conformance/core-09/stress-cache-cold-disagrees.cp`

### stress/U27: one spelling for "the slow version of"
Need: write the brute without wondering which decorator it is.
Design: `@alt(f)` and `@alt(program)` only.
Milestone: M1-differentiating. Status: supported
Evidence: conformance `spec/conformance/core-06/brute-fn-is-removed.cp`

### stress/U28: stress passes but the judge says WA at maximum size
Need: find the overflow or limit bug that only maximum-size input reaches, when small cases all agree.
Design: the clean report points at `cpl lint`, whose bound lints (L0306, L0302, L0599, L0305) each name a witness input, and stress takes that witness as a case.
Milestone: M1-critical. Status: supported (each witness, the rest of its input generated, runs before the generated cases; a failing one reads `FAIL on the input `cpl lint` names at the bounds, saved as <file>.tests/lint-<code>-<line>.in`, the file `cpl lint --save` writes and `cpl test` runs; a passing one reads `the input `cpl lint` names at the bounds passes`, and one stress can't make is named with the reason)
Evidence: `crates/cpl-cli/tests/lint_witnesses.rs`, `crates/cpl-cli/tests/wrong_answer.rs` `stress_runs_the_input_a_bound_lint_names`

### stress/U29: the interpreter and the judge's C++ disagree
Need: know that a difference is a toolchain bug, not a WA, and under which judge profile stress ran.
Design: the header names the build and profile (widths, stack); a case that differs only between builds is reported as a toolchain bug.
Milestone: M1-critical. Status: supported (the header reads `build: judge `codeforces`: g++ ...; checked, as `cpl run``; a WA the debug interpreter doesn't reproduce, printing the brute's answer, reads `toolchain bug:` and points at `cpl report`, and `--format json` marks it `"toolchain_bug":true`; a planted g++ that miscompiles one line drives both; the JSON report doesn't name the build)
Evidence: integration `crates/cpl-cli/tests/wrong_answer.rs` (`stress_names_the_build_its_cases_ran`, `a_build_the_interpreter_disagrees_with_is_a_toolchain_bug`), unit `crates/cpl-stress/src/judge.rs` (`only_an_interpreter_that_agrees_with_the_brute_blames_the_toolchain`)

### stress/U30: the checker, interactor or generator fails
Need: know which helper broke and why, not see a failing solution.
Design: each helper is blamed by name with its trap rendered as a brute's is (U18), and the case is not saved as a test of the solution.
Milestone: M1-critical. Status: supported (an index, overflow, range, `None`, `todo` or stack trap is the helper's own bug: `checker RE` or `interactor RE` with its trap, not saved; a `@gen` fn's failure names its line, `failed on seed 1 at line 7: ...`; a `@check` or `@interactor` fn's trap is the `"error"` diagnostic, with code and span, in `--format json`, which `cpl judge` hands stress through the file `CPL_JUDGE_DIAGNOSTIC` names, since the solution reads the interactor's stdout)
Evidence: integration `crates/cpl-cli/tests/wrong_answer.rs` (`a_broken_helper_is_blamed_with_its_trap_not_the_solution`), `crates/cpl-stress/tests/in_process.rs` (`a_trapping_checker_in_process_is_a_checker_re`)

### stress/U31: how far a clean run reached
Need: decide whether to trust `1000 cases agree`.
Design: the clean report adds the largest size and the extreme values tried: `tried: n from 1 to 8, a up to 8 long from -1000000000 to 1000000000`.
Milestone: M1-differentiating. Status: supported (a `@gen` fn's or `--gen` program's inputs have no field names, so they read `inputs of up to 9 numbers from 1 to 40`; `--format json` carries the line as `"tried"`)
Evidence: integration `crates/cpl-cli/tests/wrong_answer.rs` (`a_clean_stress_says_how_far_its_cases_reached`)

### stress/U32: stress for as long as I can spare
Need: stop on a wall-clock budget that fits a contest, not only after N cases.
Design: `--time 10s` runs cases until the budget ends, then reports as usual: `2400 cases agree in 10s`.
Milestone: M1-differentiating. Status: supported (with `--cases` too, whichever comes first; shrinking a failure is not counted)
Evidence: integration `crates/cpl-cli/tests/wrong_answer.rs` (`stress_runs_for_the_time_it_is_given`)

### stress/U33: helper warnings stay out of the way
Need: a lint warning in the brute or interactor never reaches the report or the judged stderr.
Design: helpers are checked for errors only; their warnings go to `cpl lint`.
Milestone: M1-critical. Status: supported (an `@alt(program)` fn, a `@check` fn, an `@interactor` fn and a `--brute` file, in text and JSON)
Evidence: integration `crates/cpl-cli/tests/wrong_answer.rs` (`helper_warnings_stay_out_of_the_stress_report`)

### stress/U34: a case that traps under `cpl run` fails stress too
Need: trust that a counterexample to an overflow or a range store is found, not wrapped into a plausible answer.
Design: stress builds with every check `cpl run` has, so the case fails as RE with the same R0202 diagnostic.
Milestone: M1-critical. Status: supported
Evidence: `crates/cpl-cli/tests/wrong_answer.rs` `stress_traps_an_overflow_as_cpl_run_does`
