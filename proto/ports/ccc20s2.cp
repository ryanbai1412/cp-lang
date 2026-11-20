// cp: 0.1 1f59ac0+dirty
// CCC 2020 S2 Escape Room: from a cell holding x you may jump to any cell
// (r, c) with r * c == x. Can you get from (1, 1) to the last cell? Search
// over the numbers: holding x leads to every number written on a cell whose
// coordinates multiply to x.
use std.graph.{bfs}

let rows: 1..=1000 = read()
let cols: 1..=1000 = read()
domain R = 1..=rows
domain C = 1..=cols
domain Num = 1..=1000000
let room: [R, C]Num = read()

var leads: [Num]List<Num> = []
for r in R:
    for c in C: leads[Num(r.label * c.label)].push(room[r, c])
let reach = bfs(x => leads[x], from: [room[R.first, C.first]])
print(if reach[Num(rows * cols)] != none: "yes" else: "no")
