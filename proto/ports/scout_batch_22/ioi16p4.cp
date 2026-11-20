// IOI 2016 Paint By Numbers: a row of n cells (n <= 200000), each known
// black 'X', known white '_' or unknown '.', and k clues (k <= 100): the
// lengths of the black blocks from left to right, with at least one white
// cell between blocks. At least one painting fits. Print the row again with
// each cell as 'X' if it is black in every painting that fits, '_' if white
// in every one, and '?' if it can be either.
//
// fits(row, clues)[i][j]: can the first i cells hold exactly the first j
// blocks? Run it on the row and on the row reversed. A cell can be white if
// some split of the blocks fits on its two sides; block j can sit at a place
// if no white is under it, no black touches it, and the rest fits on both
// sides. The places it can sit are added as ranges and read back once.
// original: corpus/submissions/4686585.cpp
let row: Str = read()
let k: Int = read()
domain Clue = 1..=k
let clues: [Clue]Int = read()
let n = row.len

/// How many of the cells before each place are known white.
fn whites_before(row: Str) -> List<Int>:
    var before = [0]
    for c in row: before.push(before.last + Int(c == '_'))
    before

fn fits(row: Str, clues: List<Int>) -> List<List<Bool>>:
    let whites = whites_before(row)
    var ok: List<List<Bool>> = []
    for i in 0..=row.len:
        var here = List.filled(clues.len + 1, false)
        for j in 0..=clues.len:
            if i == 0: here[j] = j == 0
            else:
                // The last cell is white, or the last block ends here.
                if row[i - 1] != 'X' and ok[i - 1][j]: here[j] = true
                if j > 0:
                    let start = i - clues[j - 1]
                    if start >= 0 and whites[i] == whites[start]:
                        if start == 0: here[j] = here[j] or j == 1
                        elif row[start - 1] != 'X' and ok[start - 1][j - 1]: here[j] = true
        ok.push(here)
    ok

let lens = [clues[c] for c in Clue]
let before = fits(row, lens)
let after = fits(row.rev, lens.rev)     // after[n - i][j]: cells i.. hold the last j blocks
let whites = whites_before(row)

var black_change = List.filled(n + 1, 0)
for j in 0..k:
    let len = lens[j]
    for start in 0..=n - len:
        let end = start + len
        if whites[end] != whites[start]: continue
        let left = if start == 0: j == 0 else: row[start - 1] != 'X' and before[start - 1][j]
        let right = if end == n: j == k - 1 else: row[end] != 'X' and after[n - end - 1][k - 1 - j]
        if left and right:
            black_change[start] += 1
            black_change[end] -= 1

var out = ""
var blacks = 0
for i in 0..n:
    blacks += black_change[i]
    var white = false
    if row[i] != 'X':
        for j in 0..=k:
            if before[i][j] and after[n - i - 1][k - j]: white = true
    out.push(if white and blacks > 0: '?' elif white: '_' else: 'X')
print(out)
