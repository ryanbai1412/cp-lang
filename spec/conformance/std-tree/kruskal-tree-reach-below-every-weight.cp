// expect: ok, output "YES"
domain V = 0..2
let es: [0..1](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5)]
let (domain N, kt) = kruskal_tree(es, w: e => es[e].w)
print(yes(kt.reach(1, 4) == kt.leaf[1]))
