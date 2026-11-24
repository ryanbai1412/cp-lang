// let: k = n; depth = n; children = n; size = n; path = n
let n: Int = read()
domain V = 0..n
let ppar: [V]?V = [V] v => if v.ord == 0: None else: Some(V(v.ord - 1))
let pt = Rooted.from_parents(ppar)
var s = 0
for _ in 0..160:
    s += pt.path(V(0), V(n - 1)).len // timed: tree.Rooted.path
for _ in 0..400:
    s += pt.centroid().ord // timed: tree.Rooted.centroid
    s += pt.centroids().len // timed: tree.Rooted.centroids
for _ in 0..16:
    for a in pt.ancestors(V(n - 1)): // timed: tree.Rooted.ancestors
        s += a.ord // timed: tree.Rooted.ancestors
print(s)
