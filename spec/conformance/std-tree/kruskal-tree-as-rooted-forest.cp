// expect: ok, output "YES 1 3\n4 5\nYES YES"
domain V = 0..4
let es: [0..3](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 2, v: 3, w: 3), (u: 1, v: 1, w: 1)]
let (domain N, kt) = kruskal_tree(es, w: e => es[e].w)
let t = kt.tree
print(yes(t.parent(kt.leaf[2]) == t.parent(kt.leaf[3])), t.size(kt.leaf[0]), t.size(t.parent(kt.leaf[0])!))
print([Int(r) for r in t.roots])
print(yes(t.root_of(kt.leaf[1]) == kt.reach(0, 5)), yes(t.parent(kt.reach(2, 3)) == None))
