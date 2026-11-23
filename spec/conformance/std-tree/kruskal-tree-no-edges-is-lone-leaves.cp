// expect: ok, output "3 YES"
domain V = 0..3
let es: [0..0](u: V, v: V, w: Int) = []
let (domain N, kt) = kruskal_tree(es, w: e => es[e].w)
print(N.count, yes(kt.bottleneck(0, 1) == None))
