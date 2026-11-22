// expect: ok, output "6"
domain V = 0..4
let a: [Pair<V>]Int = [Pair<V>] x => 1
print(a.sum())
