// Source: CCC '20 S2 (Escape Room), https://dmoj.ca/problem/ccc20s2
// Problem: an M x N grid (M, N <= 1000) of values <= 1e6. From a cell holding x you may
//   jump to any cell (r, c) with r * c = x. Print yes if you can reach (M, N) from (1, 1),
//   else no.
// Solution: search over the numbers: holding x leads to every value written on a cell
//   whose coordinates multiply to x, and the exit is reached once a held value is M * N.
//   Bucket the cell values by r * c and BFS from the value at (1, 1). O(MN + 1e6).
// Bounds: rows, cols: 1..=1000; room: 1..=1000000

// original: corpus/submissions/4323590.cpp
with io(answers: ("yes", "no"))
let rows: 1..=1000 = read()
let cols: 1..=1000 = read()
domain R = 1..=rows
domain C = 1..=cols
domain Num = 1..=1000000
let room: [R, C]Num = read()

var leads: [Num]List<Num> = []
for r in R:
    for c in C: leads[Num(r.label * c.label)].push(room[r, c])
let exit = Num(rows * cols)
let reach = bfs(x => leads[x], from: [room[R.first!, C.first!]], until: exit)
print(yes(reach.reached(exit)))
