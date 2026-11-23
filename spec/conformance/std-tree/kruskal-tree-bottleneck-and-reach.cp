// expect: ok, output "5 YES YES YES"
domain V = 0..3
let es: [0..2](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 1, v: 2, w: 3)]
let (domain N, kt) = kruskal_tree(es, w: e => es[e].w)
print(N.count, yes(kt.bottleneck(0, 2) == Some(5) and kt.bottleneck(1, 2) == Some(3) and kt.bottleneck(0, 0) == None), yes(kt.reach(0, 4) == kt.leaf[0]), yes(kt.w[kt.reach(1, 4)] == Some(3) and kt.w[kt.reach(1, 5)] == Some(5)))
