# Conformance examples

Programs that must compile and run, and programs that must be rejected
with a given diagnostic. One file per case, grouped by chapter
(`core-05/`, `lib-ops-ranges/`, ...). The first line states the
expectation:

```
// expect: ok
// expect: ok, output "3"
// expect: E0601
// expect: L0601
// expect: R0301 (checked)
```

A code names no line number: the line the diagnostic or trap points at
ends with the marker `// <- here`, and exactly one line carries it
(decision 506). Header lines can come and go without moving it. A failing
case names the line its diagnostic is on; `scripts/spec-gate.py markers
--fix` rewrites an old `at N` case. A second header line `// rule: 05.4.2 05.4.3`
names the numbered rules (`(R05.4.2)` in `spec/core`) the case tests,
without the `R`; `cpl-conform --rules` lists the rules no case cites.
A rule is defined where `(Rnn.n.n)` ends its sentence, or `(Rnn.n.n, permits)` when its text
matches a forbidding word but only permits or defines something, so it needs no reject case; a reference to a
rule from elsewhere is written `(see Rnn.n.n)` so it is not read as a
second definition. `COVERAGE.md` (from `scripts/spec-gate.py coverage`)
lists per chapter the rules, codes and std items each case covers. An `R` code
expects the program to check, run and trap with that code on that line.
An `L` code runs `cpl lint` (default levels) after the check. An optional
header line `// input: "3\n1 2 3\n"` gives the run its standard input,
escaped like `output "..."`; without it the input is empty.
A case whose run must exhaust a budget before it traps (deep recursion)
may raise its run's time limit with `// timeout: <secs>`; a value below the
runner's default (`CPL_CONFORM_CASE_TIMEOUT`, 20 s) has no effect.

A case for a rule only a tool can show names the tool in a header line
`// tool: build -o -`: the runner copies the case to a scratch directory
and runs `cpl build <copy> -o -` there, twice, and fails the case when
the two runs differ (tool output is deterministic). The `// expect:`
line then reads against that run: `ok` is exit status 0, `ok, output
"..."` also compares standard output, a code must appear with
the marked line in the output, and `exit N` (tool cases only) wants that exit
status (`spec/tools/cli.md`). Each `// has: "..."` line names text the
output must contain and each `// lacks: "..."` line text it must not;
each `// rewritten: "..."` line (tool cases only) text the case file
must contain after the run, so a case can check the edits `cpl fix` or
`cpl fmt` made. The copy blanks those three kinds of line so a tool that
echoes the source can't match them. Each `// file: <path> "..."` line (tool cases only)
writes one more file, escaped like `// input:`, at that path below the
scratch directory before the run: a `cp.toml` and a library module, say.

A `// copies: N` header line states how many deep copies the case's
judge C++ makes (none without the line; `N` above 0, at most one line).
`crates/cpl-cli/tests/read_only_copies.rs` builds every case and every
corpus program and fails, naming the file, on any whose C++ makes a
different number; `scripts/debt-ratchet.py` sums the lines as the
shrink-only `cpp_deep_copies` counter, so a copy is only added by writing
it down where it happens.

A word holding `{case}` or `{dir}` takes the copy's path or the scratch
directory there, and the copy is not passed first:
`// tool: setup neovim --dir {dir}/nvim`. `// tool: lsp <feature>` opens the
copy in `cpl lsp --stdio` and prints what the server sends, one JSON
message per line, with the document's URI as `<case>`: `diagnostics` (the
published diagnostics), `formatting`, and `hover`, `completion` or
`definition` at a needle's `|`, as in `// tool: lsp completion h.|` (the
first non-comment line holding `h.`). `// tool: dap <step>...` launches the
copy in `cpl dap` and prints every response and event, one JSON message per
line, with its path as `<case>`; the steps are `break:N` (line N, counted from after the
header lines, as in `// expect: X at N`),
`continue`, `next`, `stepIn`, `stepOut`, `stepBack`, `reverseContinue` and
`eval:EXPR`. Each `// before: <tool args>` line runs that tool first in the
same scratch directory, as `// tool:` would, and must exit 0: `// before: run
--trace` writes the trace a `// tool: replay` case reads.

They double as the spec's exercises and as the regression suite: the
runner (`cpl-conform conformance`, in `crates/cpl-conform`) checks each expectation
against `cpl check`/`cpl run`, and reports which cases 0.2.x does not
meet yet without failing the fast check. Seeded from the exercises in
`core/05-ownership.md`.
