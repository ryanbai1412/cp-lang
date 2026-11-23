// expect: ok
// rule: 03.2.7
// tool: build -o -
// has: "std::vector<cpl::OptInt> a"
// lacks: "std::bitset"
// lacks: "Bits<"
domain V = read(1..=100)
var a: [V]?Int = None
let i = read<V>()
a[i] = Some(3)
print(a[i] ?? 0)
