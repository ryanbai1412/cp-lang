// expect: exit 1
// rule: judge.7
// tool: check {case}
// file: cp.toml "[judge.small]\nextends = \"generic\"\nmemory_mb = 1\n"
// has: "E0904"
// has: "over the 1 MB limit on small: `a` 8 MB"
// judge: small
// The peak at the input bounds is checked against the judge's `memory_mb`: an 8 MB array on a 1 MB judge is E0904, naming the judge and the array.
var a: [0..1000000]Int = 0
a[5] = 1
print(a[5])
