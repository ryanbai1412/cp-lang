// UCRPC 2021 F, tuned: port.cp's Dijkstra written out over a plain distance table, skipping stale
// heap entries as the original does, instead of `dijkstra` keeping an optional distance, a done
// flag and the settle order per cell.
let (n, t): (1..=1000, 0..=1000) = read()
domain Row = [n]
domain Col = [n]
let g: [Row, Col]Char = read()

var dist: [Row, Col]Int = Int.MAX
var h: Heap<(Int, (Row, Col))> = Heap()
let start = (Row.first!, Col.first!)
dist[start] = 0
h.push((0, start))
while let (d, p) = h.pop():
    if d > dist[p]: continue
    for q in p.adj4:
        let nd = d + (if g[q] == 'b': 1 + t else: 1)
        if nd < dist[q]:
            dist[q] = nd
            h.push((nd, q))
print(dist[Row.last!, Col.last!])
