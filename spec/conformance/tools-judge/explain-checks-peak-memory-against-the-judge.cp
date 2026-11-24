// expect: ok
// rule: judge.7
// tool: explain {case}
// file: cp.toml "[judge.small]\nextends = \"generic\"\nmemory_mb = 1\n"
// has: "1 MB"
// judge: small
// `cpl explain` checks peak memory against the judge's `memory_mb`: an 8 MB array on a 1 MB judge is reported against that limit, not just listed.
var a: [0..1000000]Int = 0
a[5] = 1
print(a[5])
