let xs = [1, 2, 3]
print(xs, sep: ", ")
let o: ?Int = none
print(o, none: "-1")
print(" ".join(xs))
print(xs.get(1) ?? -1, xs.get(7) ?? -1)
let os: List<?Int> = [1, none, 3]
print(os, none: "x")
print(3, xs, sep: "\n")
domain D = 0..3
let a: [D]Int = [7, 8, 9]
print(",".join(a), "-".join(["ab", "c"]))
domain V = 1..=4
let g = Graph.undirected([(V(1), V(2)), (V(2), V(3))])
for u in V: print(u, ":", g[u])
print(fixed(1, 3, 5), fixed(-1, 300, 2), fixed(7, 2, 0), "{-7:05}|{255:x}|{5:b}|{-0.0004:.2}|{3:6.2f}")
