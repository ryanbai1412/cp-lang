// expect: ok
// rule: 05.3.5
// tool: build -o -
// lacks: "std::tuple<ll, std::vector<ll>> "
// `for (k, v) in m` over a `Map<K, List<T>>` lends each entry; it must not copy the list into a tuple (round 26, B4).
let at = (0..6).group(by: i => i % 2)
var t = 0
for (k, v) in at: t += k * v.len
print(t)
