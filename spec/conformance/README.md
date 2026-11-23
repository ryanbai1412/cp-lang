# Conformance examples

Programs that must compile and run, and programs that must be rejected
with a given diagnostic. One file per case, grouped by chapter
(`core-05/`, `lib-ops-ranges/`, ...). The first line states the
expectation:

```
// expect: ok
// expect: ok, output "3"
// expect: E0601 at 4
// expect: L0601 at 3
// expect: R0301 at 2 (checked)
```

Line numbers count from the first line after the header, so `at 1` is the
first line of the program. A second header line `// rule: 05.4.2 05.4.3`
names the numbered rules (`(R05.4.2)` in `spec/core`) the case tests,
without the `R`; `cpl-conform --rules` lists the rules no case cites.
A rule is defined where `(Rnn.n.n)` ends its sentence; a reference to a
rule from elsewhere is written `(see Rnn.n.n)` so it is not read as a
second definition. `COVERAGE.md` (from `scripts/conformance-coverage.py`)
lists per chapter the rules, codes and std items each case covers. An `R` code
expects the program to check, run and trap with that code on that line.
An `L` code runs `cpl lint` (default levels) after the check. An optional
header line `// input: "3\n1 2 3\n"` gives the run its standard input,
escaped like `output "..."`; without it the input is empty.

A case for a rule only a tool can show names the tool in a header line
`// tool: build -o -`: the runner copies the case to a scratch directory
and runs `cpl build <copy> -o -` there, twice, and fails the case when
the two runs differ (tool output is deterministic). The `// expect:`
line then reads against that run: `ok` is exit status 0, `ok, output
"..."` also compares standard output, a code `at N` must appear with
its line in the output, and `exit N` (tool cases only) wants that exit
status (`spec/tools/cli.md`). Each `// has: "..."` line names text the
output must contain and each `// lacks: "..."` line text it must not;
the copy blanks those two kinds of line so a tool that echoes the source
can't match them. Each `// file: <path> "..."` line (tool cases only)
writes one more file, escaped like `// input:`, at that path below the
scratch directory before the run: a `cp.toml` and a library module, say.

They double as the spec's exercises and as the regression suite: the
runner (`cpl-conform conformance`, in `crates/cpl-conform`) checks each expectation
against `cpl check`/`cpl run`, and reports which cases 0.2.x does not
meet yet without failing the fast check. Seeded from the exercises in
`core/05-ownership.md`.
