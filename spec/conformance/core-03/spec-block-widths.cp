// expect: ok, output "3 6"
// rule: 03.5.1
// input: "3\n1 3 2\n"
let n = read<1..=2e5>()
domain V = 0..n
let a = read<[V]1..=1e9>()           // 4 B per element
var par: [V]?V = [V].filled(None)    // 4 B, None = all ones
var dist: [V]?Int = [V].filled(None) // 8 B, None = INT64_MIN
var mx = SegTree(max, a)             // 4 B nodes: max is Selective
var sm = SegTree(+, a)               // 8 B nodes: sums are Int
struct Item: w: 1..=1e9, v: 1..=1e9, id: V    // 12 B, ?Item 12 B
print(mx.fold(V.range), sm.fold(V.range))
