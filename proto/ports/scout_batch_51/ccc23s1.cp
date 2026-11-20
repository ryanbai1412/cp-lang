// CCC '23 S1 (Trianglane): perimeter of wet tiles on a 2xN alternating triangle strip.
// Uses v2.1's `Bool` Index directly for the two rows (`[Bool, Col]0..=1`, `for r in Bool`,
// `wet[not r, c]`) and optional stepping `c - 1` / `c + 1` for horizontal neighbours.
// original: corpus/submissions/5350804.cpp

let n: 1..=200_000 = read()
domain Col = 0..n
let wet: [Bool, Col]0..=1 = read()

var ans = 0
for r in Bool:
    for c in Col if wet[r, c] == 1:
        ans += 3
        if let p = c - 1 and wet[r, p] == 1: ans -= 1
        if let q = c + 1 and wet[r, q] == 1: ans -= 1
        if c.to_idx % 2 == 0 and wet[not r, c] == 1: ans -= 1
print(ans)
