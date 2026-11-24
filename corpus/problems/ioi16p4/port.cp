// Source: IOI 2016 P4 (Paint By Numbers), https://dmoj.ca/problem/ioi16p4
// Problem: a row of n <= 200000 cells, each known black 'X', known white '_' or unknown '.',
//   and k <= 100 clues: the lengths of the black blocks from left to right. At least one
//   painting fits. Print each cell as 'X' (black in every fitting painting), '_' (white in
//   every one) or '?'. Input: the row, k, then the clues.
// Solution: fits[i][j]: can the first i cells hold exactly the first j blocks; run it on the
//   row and on the reversed row (blocks reversed). A cell can be white if some split j fits
//   on both sides of it; block j can start at s if no known white lies under it, the cells
//   beside it are not 'X' and both sides fit. Every such placement is a +1/-1 range in a
//   difference array, so a cell can be black when its prefix sum is positive. O(nk).
// original: corpus/submissions/4686585.cpp
let row = read<Str of "X_.">()
let n = row.len
assert 1 <= n <= 200_000
let k: 1..=100 = read()
let clues = read<[k]1..=n>()

// whites_before(row)[i]: the known white cells among the first i
fn whites_before(row: Str) -> [0..=n]Int:
    var count: [0..=n]Int = 0
    for (i, c) in row.items: count[i + 1] = count[i] + Int(c == '_')
    count

// fits(row, lens)[i, j]: the first i cells can hold exactly the first j blocks
fn fits(row: Str, lens: List<Int>) -> [0..=n, 0..=k]Bool:
    let blanks = whites_before(row)
    var ok: [0..=n, 0..=k]Bool = false
    ok[0, 0] = true
    for i in 1..=n:
        for j in 0..=k:
            // the last cell is white, or block j ends here, right after a white (or the start)
            let blank = row[i - 1] != 'X' and ok[i - 1, j]
            let start = if j > 0: i - lens[j - 1] else: -1
            let block = start >= 0 and blanks[i] == blanks[start]
                and (if start == 0: j == 1 else: row[start - 1] != 'X' and ok[start - 1, j - 1])
            ok[i, j] = blank or block
    ok

let lens = clues.collect()
let before = fits(row, lens)
let after = fits(row.rev, lens.rev)  // after[n - i, j]: cells i.. hold the last j blocks
let whites = whites_before(row)

// every placement of block j at start s that fits both sides paints s..s + len black
domain Cell = 0..n
var black = Diff<Cell, Int>()
for j in 0..k:
    let len = lens[j]
    for start in 0..=n - len:
        let end = start + len
        let left = if start == 0: j == 0 else: row[start - 1] != 'X' and before[start - 1, j]
        let right = if end == n: j == k - 1 else: row[end] != 'X' and after[n - end - 1, k - 1 - j]
        if whites[end] == whites[start] and left and right: black.add(Cell(start)..=Cell(end - 1), 1)
let can_black = black.finish()

let white = (c: Cell) => row[Int(c)] != 'X' and (0..=k).any(j => before[Int(c), j] and after[n - Int(c) - 1, k - j])
print(Str(Cell.map(c => if white(c) and can_black[c] > 0: '?' elif white(c): '_' else: 'X')))
