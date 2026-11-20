// CCC 2020 S2 Escape Room: from a cell holding x you may jump to any cell
// (r, c) with r * c == x. Can you get from (1, 1) to (rows, cols)? Search over
// the numbers: x leads to every number written on a cell whose coordinates
// multiply to x.
use std.graph.{bfs}

let (rows, cols): (1..=1000, 1..=1000) = read()
domain Num = 1..=1e6
let room: [0..rows, 0..cols]Num = read()

var leads: [Num]List<Num> = []
for r in 0..rows:
    for c in 0..cols: leads[Num((r + 1) * (c + 1))].push(room[r, c])
let jumps = bfs(x => leads[x], from: [room[0, 0]])
print(if jumps[Num(rows * cols)] != none: "yes" else: "no")
