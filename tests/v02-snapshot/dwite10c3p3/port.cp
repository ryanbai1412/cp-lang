// Source: DWITE '10 R3 #3 (Dominos Tiling), https://dmoj.ca/problem/dwite10c3p3
// Problem: five lines, each a width 0 <= N <= 30; for each print the number of domino tilings
//   of a 3 x N board, modulo 10^6.
// Solution: DP column by column over the set of the column's cells already covered by dominoes
//   lying in from the left: choose which free cells start a lying domino into the next
//   column; the rest must pair up vertically (none, the top two or the bottom two).
//   O(N * 8 * 8) per width.
// original: corpus/submissions/4804312.cpp
const MOD = 1_000_000
domain Cell = 0..3  // the cells of one column, top to bottom
type Cells = Subset<Cell>
let upper = Cells.empty.with(Cell(0)).with(Cell(1))
let lower = Cells.empty.with(Cell(1)).with(Cell(2))

fn stands(c: Cells) -> Bool:  // can upright dominoes cover exactly these?
    c == Cells.empty or c == upper or c == lower

for _ in 0..5:
    let n: 1..=30 = read()
    var ways: [Cells]Int = 0  // by the set already filled
    ways[Cells.empty] = 1
    for _ in 0..n:
        var next: [Cells]Int = 0
        for filled in Cells:
            let free = Cells.full - filled
            for lying in free.subsets:
                if stands(free - lying): next[lying] = (next[lying] + ways[filled]) % MOD
        ways = next
    print(ways[Cells.empty])
