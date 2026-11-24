// expect: ok
// rule: judge.9
// tool: build --judge fast -o -
// file: cp.toml "[judge.fast]\nextends = \"generic\"\nisa = \"avx2\"\n"
// has: "#include <vector>"
// has: "#pragma GCC target(\"avx2\")"
// has: "#pragma GCC optimize(\"O3,unroll-loops,fp-contract=off\")"
// A judge whose CPUs have AVX2 gets target pragmas, after the includes.
let n: 1..=100 = read()
print(n * 2)
