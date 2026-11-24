// Source: NOIP 2008 P3 (Message Passing), https://dmoj.ca/problem/noip08p3
// limits: 1000 ms, 255 MB
// Problem: an m x n class (m, n <= 50), each pupil with kindness 0..=100 (the two corners 0).
//   A note goes from the top-left to the bottom-right moving down or right, and a reply comes
//   back moving up or left, with no pupil used by both. Print the largest total kindness.
// Solution: reversed, the reply is a second down/right path, so walk both together one step at
//   a time: after d steps a path in row r is in column d - r, so the state is d and the two
//   rows a < b (equal only at the corners; ordered rows lose nothing, as crossing paths can swap
//   tails). @memo best(d, a, b) is the most kindness still to collect, over the 4 move pairs.
//   O((m + n) m^2) states, O(1) each.
// original: corpus/submissions/5185022.cpp
let n: 1..=50 = read()
let m: 1..=50 = read()
let kind: [n, m]0..=100 = read()
let last = n + m - 2  // steps from corner to corner

// the notes may share a seat only at the two corners
fn apart(d: Int, a: Int, b: Int) -> Bool:
    a < b or (a == b and (d == 0 or d == last))

// most kindness still to collect with the notes in rows a and b after d steps
@memo
fn best(d: 0..=last, a: 0..n, b: 0..n) -> ?Int:
    if d == last: return 0
    var most: ?Int = None
    for a2 in a..=a + 1:
        for b2 in b..=b + 1:
            if b2 < n and d + 1 - a2 < m and apart(d + 1, a2, b2):
                if let rest = best(d + 1, a2, b2):
                    most max= rest + kind[a2, d + 1 - a2] + kind[b2, d + 1 - b2]
    most

print(best(0, 0, 0) ?? 0)
