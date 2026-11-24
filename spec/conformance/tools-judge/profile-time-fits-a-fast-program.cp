// expect: ok
// rule: judge.6
// tool: test {case}
// file: cp.toml "[judge.slow]\nextends = \"generic\"\ntime_ms = 5000\n"
// file: profile-time-fits-a-fast-program.tests/1.in "\n"
// file: profile-time-fits-a-fast-program.tests/1.out "3\n"
// has: "1/1 passed"
// lacks: "TLE"
// judge: slow
// A problem with no `[limits]` runs under its judge profile's `time_ms`; a program well inside it passes.
print(1 + 2)
