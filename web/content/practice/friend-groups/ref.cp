let (n, m): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
var d = Dsu<V>.new()
var groups = n
var big = 1
for _ in 0..m:
    let (a, b): (V, V) = read()
    if d.unite(a, b): groups -= 1
    big max= d.size(a)
    print(groups, big)
