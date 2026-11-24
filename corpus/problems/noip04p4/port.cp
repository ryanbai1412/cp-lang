// Source: NOIP 2004 P4, Alphametic Puzzle, https://dmoj.ca/problem/noip04p4
// limits: 1000 ms, 256 MB
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
domain Digit = 0..n
domain Col = 0..n  // units first
let words: [0..3, Col]Char = read()
let at = [0..3, Col] (i, c) => Letter(words[i, Col(n - 1 - c.ord)] - 'A')

var digit: [Letter]?Digit = None
var used: [Digit]Bool = false

fn set(l: Letter, d: ?Digit):
    if let old = digit[l]: used[old] = false
    digit[l] = d
    if let new = d: used[new] = true

// no fully known column contradicts a + b (+ carry) == c in base n
fn plausible() -> Bool:
    all(not (if let x = digit[at[0, c]] and let y = digit[at[1, c]] and let z = digit[at[2, c]]:
            (x.ord + y.ord) % n != z.ord and (x.ord + y.ord + 1) % n != z.ord
        else: false) for c in Col)

// fix the columns from the units up: guess the two addend letters, then the
// sum letter is forced
fn solve(col: ?Col, carry: 0..=1) -> Bool:
    let c = col else: return carry == 0
    for i in 0..2:
        let l = at[i, c]
        if digit[l] != None: continue
        for d in Digit.rev if not used[d]:
            set(l, d)
            if plausible() and solve(col, carry): return true
            set(l, None)
        return false
    let total = digit[at[0, c]]!.ord + digit[at[1, c]]!.ord + carry
    let (l, z) = (at[2, c], Digit(total % n))
    if digit[l] == z: return solve(c.next, total / n)
    if digit[l] != None or used[z]: return false
    set(l, z)
    if solve(c.next, total / n): return true
    set(l, None)
    false

if solve(Col.first, 0): print([d!.ord for d in digit])
