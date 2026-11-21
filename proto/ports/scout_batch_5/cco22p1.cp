// CCO 2022 Alternating Heights: students stand in a line, each from one of k
// height classes. For each query x, y: can the classes be given distinct
// heights so that the line from x to y goes up, down, up, ...?
// original: corpus/submissions/4801881.cpp
// WISH F1

let n: 2..=3000 = read()
let k: 2..=n = read()
let q: 1..=1e6 = read()
domain Spot = 1..=n
domain Student = 1..=k
let line: [Spot]Student = read()
let asks: [q]Span<Spot> = read()  // WISH K19: an ordered pair, read and generated as one

// Every neighbouring pair says "this class is shorter than that one"; the
// stretch works exactly when those claims have no cycle.
fn consistent(x: Spot, y: Spot) -> Bool:
    var taller: [Student]List<Student> = []
    for ((i, j), rising) in zip((x..=y).pairs, cycle([true, false])):  // WISH G21, K14
        if rising: taller[line[i]].push(line[j])
        else: taller[line[j]].push(line[i])
    topo(s => taller[s]) != none

// A stretch of a consistent stretch is consistent, so each start has a furthest end.
let reach = [Spot] x => (x..).last_where(y => consistent(x, y))!  // WISH K6
for (x, y) in asks: print(if y <= reach[x]: "YES" else: "NO")
