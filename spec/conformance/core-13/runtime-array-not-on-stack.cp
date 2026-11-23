// expect: ok
// rule: 13.2.9 02.8.5
// tool: build -o -
// has: "std::vector<ll> b"
// lacks: "alloca"
// lacks: "ll b["
domain V = read(1..=100)
var b: [V]Int = 0
let i = read<V>()
b[i] = 4
print(b[i])
