// expect: ok
// rule: judge.9
// tool: build --judge far -o -
// file: cp.toml "[judge.far]\nextends = \"generic\"\nisa = \"unknown\"\n"
// lacks: "#pragma GCC target"
// lacks: "O3"
// A judge whose CPUs are unknown, or not x86, is promised no feature, so its C++ asks for none.
let n: 1..=100 = read()
print(n * 2)
