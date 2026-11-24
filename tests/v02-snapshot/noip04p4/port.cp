// Source: NOIP 2004 P4, Alphametic Puzzle, https://dmoj.ca/problem/noip04p4
// Problem: base n <= 26; three words of n letters over the first n capital letters, a + b = c,
//   each letter a distinct digit 0..n-1. The solution is unique; print the digit of each letter
//   A, B, C, ... in order.
// Solution: backtrack column by column from the units with the carry: guess unknown addend
//   letters (largest digits first), then the sum letter is forced. After every guess
//   `plausible` rejects any fully known column where neither a + b nor a + b + 1 gives c mod n,
//   which prunes the search hard. Exponential worst case, fast in practice.
// original: corpus/submissions/5117252.cpp
let n: 1..=26 = read()
domain Letter = 0..n
let words: [0..3, 0..n]Char = read()
// rows[i][col]: the letter of addend / addend / sum at column col, units first
let rows = [[Letter(words[i, n - 1 - col] - 'A') for col in 0..n] for i in 0..3]

var digit: [Letter]Int = -1
var used: [0..n]Bool = false

// no fully known column contradicts a + b (+ carry) == c in base n
fn plausible() -> Bool:
    for col in 0..n:
        let (x, y, z) = (digit[rows[0][col]], digit[rows[1][col]], digit[rows[2][col]])
        if x >= 0 and y >= 0 and z >= 0 and (x + y) % n != z and (x + y + 1) % n != z: return false
    return true

// fix the columns from the units up: guess the two addend letters, then the
// sum letter is forced
fn solve(col: Int, carry: Int) -> Bool:
    if col == n: return carry == 0
    for i in 0..2:
        let l = rows[i][col]
        if digit[l] >= 0: continue
        for d in (0..n).rev:
            if used[d]: continue
            digit[l] = d
            used[d] = true
            if plausible() and solve(col, carry): return true
            digit[l] = -1
            used[d] = false
        return false
    let total = digit[rows[0][col]] + digit[rows[1][col]] + carry
    let l = rows[2][col]
    if digit[l] == total % n: return solve(col + 1, total / n)
    if digit[l] >= 0 or used[total % n]: return false
    digit[l] = total % n
    used[total % n] = true
    if solve(col + 1, total / n): return true
    digit[l] = -1
    used[total % n] = false
    return false

let solved = solve(0, 0)
assert solved
print(digit)
