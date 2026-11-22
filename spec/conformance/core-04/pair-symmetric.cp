// expect: ok, output "6\n7"
// rule: 04.2.12
domain V = 1..=4
print(Pair<V>.count)
var d: [Pair<V>]Int = 0
d[1, 3] = 7
print(d[3, 1])
