// expect: ok
// rule: judge.7
// tool: explain {case}
// file: cp.toml "[judge.small]\nextends = \"generic\"\nmemory_mb = 16\n"
// has: "of 16 MB on small"
// judge: small
// `cpl explain` checks peak memory against the judge's `memory_mb`: an 8 MB array on a 16 MB judge is shown against that limit, naming the judge.
var a: [0..1000000]Int = 0
a[5] = 1
print(a[5])
