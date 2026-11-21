let (n, m): (Int, Int) = read()
domain V = 0..n
var dsu = Dsu<V>()
var comps = n
for _ in 0..m:
    let (u, v): (V, V) = read()
    if dsu.unite(u, v):
        comps -= 1
print(comps)
