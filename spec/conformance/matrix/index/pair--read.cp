// expect: ok, output "6"
// input: "1 1 1 1 1 1"
domain V = 0..4
let a: [Pair<V>]Int = read()
print(a.sum())
