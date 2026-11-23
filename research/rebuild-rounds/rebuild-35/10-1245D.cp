// CF 1245D Shichikuji and Power Grid: MST with a virtual vertex 0 joined to every city by c.
let n: 1..=2000 = read()
domain V = 0..=n
let pt = read<[n](x: Int, y: Int)>()
let c = read<[n]Int>()
let k = read<[n]Int>()
var es: List<(u: V, v: V, w: Int)> = []
for i in 0..n:
    es.push((u: V(0), v: V(i + 1), w: c[i]))
    for j in i + 1..n:
        let len = abs(pt[i].x - pt[j].x) + abs(pt[i].y - pt[j].y)
        es.push((u: V(i + 1), v: V(j + 1), w: (k[i] + k[j]) * len))
domain E = 0..es.len
let ea: [E](u: V, v: V, w: Int) = [E] e => es[e.ord]
let mst = kruskal(ea)
let plants = [ea[e].v for e in E if mst.take[e] and ea[e].u == V(0)]
let wires = [ea[e] for e in E if mst.take[e] and ea[e].u != V(0)]
print(mst.total)
print(plants.len)
print(plants)
print(wires.len)
for r in wires: print(r.u, r.v)
