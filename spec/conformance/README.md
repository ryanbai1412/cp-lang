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
names the numbered rules (`(R05.4.2)` in `spec/core`) the case tests;
`scripts/conformance.py --rules` lists the rules no case cites. An `R` code
expects the program to check, run and trap with that code on that line.

They double as the spec's exercises and as the regression suite: the
runner (`scripts/conformance.py`) checks each expectation
against `cpl check`/`cpl run`, and reports which cases 0.2.x does not
meet yet without failing the fast check. Seeded from the exercises in
`core/05-ownership.md`.
