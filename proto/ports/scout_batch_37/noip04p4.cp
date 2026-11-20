// NOIP 2004 P4, Alphametic Puzzle
// original: corpus/submissions/5117252.cpp
let n: 1..=26 = read()
domain Letter = 0..n
let words: [3, 0..n]Char = read()
// rows[i, col]: the letter of addend / addend / sum at column col, units first
let rows = [0..3, 0..n] (i, col) => Letter(words[i, n - 1 - col] - 'A')

var digit: [Letter]?Int = none
var used: [0..n]Bool = false

// no fully known column contradicts a + b (+ carry) == c in base n
fn plausible() -> Bool:
    for col in 0..n:
        if let x = digit[rows[0, col]] and let y = digit[rows[1, col]] and let z = digit[rows[2, col]]:
            if (x + y) % n != z and (x + y + 1) % n != z: return false
    return true

// fix the columns from the units up: guess the two addend letters, then the
// sum letter is forced
fn solve(col: Int, carry: Int) -> Bool:
    if col == n: return carry == 0
    for l in [rows[0, col], rows[1, col]] if digit[l] == none:
        for d in (0..n).rev if not used[d]:
            digit[l] = d
            used[d] = true
            if plausible() and solve(col, carry): return true
            digit[l] = none
            used[d] = false
        return false
    let total = digit[rows[0, col]]! + digit[rows[1, col]]! + carry
    let l = rows[2, col]
    if digit[l] == total % n: return solve(col + 1, total / n)
    if digit[l] != none or used[total % n]: return false
    digit[l] = total % n
    used[total % n] = true
    if solve(col + 1, total / n): return true
    digit[l] = none
    used[total % n] = false
    return false

let solved = solve(0, 0)
assert solved
print(digit[l]! for l in Letter)
