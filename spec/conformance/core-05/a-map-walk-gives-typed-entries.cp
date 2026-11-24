// expect: ok, output "0 1\n1 4\n7\n17"
// rule: 05.3.5
// `for (k, v) in m` walks a `Map` or `HashMap` by its entries as `(K, V)`, so both names carry their types: `group.len` is a member of a `List<Int>`, and `k + v` adds two `Int`s.
let xs = [3, 1, 4, 1, 5]
for (g, group) in xs.group(by: x => x % 2):
    print(g, group.len)
var m: Map<Int, Int> = Map()
m[3] = 4
var t = 0
for (k, v) in m: t += k + v
print(t)
var h: HashMap<Int, Int> = HashMap()
h[2] = 5
for (k, v) in h: t += k * v
print(t)
