// expect: ok
// rule: judge.9
// tool: build --judge fast -o -
// file: cp.toml "[judge.fast]\nextends = \"generic\"\nisa = \"avx2\"\n"
// has: "popcnt"
// has: "__builtin_popcountll"
// An AVX2 judge's CPUs have popcnt, so its target pragma names it and `popcount` is one instruction, not a library call.
let n: Int = read()
print(popcount(n))
