// Source: IOI 2001 Day 1 P3 (Twofive), https://dmoj.ca/problem/ioi01p3
// limits: 100 ms, 32 MB
// Problem: words are the 25 letters A..Y written row by row in a 5x5 grid with every row and
//   column increasing, numbered from 1 in lexicographic order (fewer than 2^31 of them). Given
//   "W" and a word print its number; given "N" and a number print that word.
// Solution: placing A..Y in order, the filled cells always form a staircase (row r shorter than
//   row r - 1), so ways(key) counts the completions from a staircase of five row lengths packed
//   base 6, memoized over 7776 keys, with cell_of[] pinning chosen letters to cells. Rank: for
//   each cell add the completions of every smaller unused letter put there; unrank: walk the
//   letters skipping whole blocks. Each count is O(6^5 * 5), at most 25 * 25 counts.
// original: corpus/submissions/4806167.cpp
// stdin: "W" and a 25-letter word (print its rank), or "N" and a rank (print the word).
let kind: Char = read()

// A grid is valid when letters A..Y, placed in order, always extend a
// staircase: row r may grow only while it stays shorter than row r - 1.
// A staircase is five row lengths, packed base 6; ways(key) counts the valid
// completions while cell_of pins chosen letters to their cells.
domain Letter = 0..25
domain Cell = 0..25
domain Key = 0..7776
var cell_of: [Letter]?Cell = None
var memo: [Key]?Int = None
let weight: List<Int> = [1296, 216, 36, 6, 1]  // weight[r]: one more cell in row r
fn ways(key: Key) -> Int:
    if key == Key.last!: return 1
    if let known = memo[key]: return known
    let len = [key.ord / weight[r] % 6 for r in 0..5]
    let next = Letter.nth(len.sum())
    var total = 0
    for r in 0..5:
        if len[r] < 5 and (r == 0 or len[r - 1] > len[r]):
            let cell = Cell.nth(r * 5 + len[r])
            if (cell_of[next] ?? cell) == cell: total += ways(Key.nth(key.ord + weight[r]))
    memo[key] = total
    total

fn completions() -> Int:
    for key in Key: memo[key] = None
    ways(Key.first!)

var used: [Letter]Bool = false
if kind == 'W':
    let word: [Cell]Char = read()
    var rank = 1
    for cell in Cell:
        let letter = Letter.nth(word[cell] - 'A')
        for smaller in Letter:
            if smaller == letter: break
            if not used[smaller]:
                cell_of[smaller] = cell
                rank += completions()
                cell_of[smaller] = None
        cell_of[letter] = cell
        used[letter] = true
    print(rank)
else:
    let rank: Int = read()
    var skip = rank - 1
    var word: List<Char> = []
    for cell in Cell:
        for letter in Letter:
            if used[letter]: continue
            cell_of[letter] = cell
            let here = completions()
            if skip < here:
                used[letter] = true
                word.push('A' + letter.ord)
                break
            skip -= here
            cell_of[letter] = None
    print(word)
