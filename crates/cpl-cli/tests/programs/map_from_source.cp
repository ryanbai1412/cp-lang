// `Map(src)` and `HashMap(src)` hold every pair `src` walks, from a map or a list of pairs.
var ways: HashMap<Int, Int> = [0: 1, 5: 2, 3: 7]
print(Map(ways).len)
let m: Map<Int, Int> = Map(ways)
print(m)
let p = Map([(1, "a"), (3, "b"), (1, "c")])
print(p.len, p[1], p[3])
let h = HashMap(m)
print(h.len, h[5])
var t = 0
for (k, v) in Map(h): t += k * v
print(t)
