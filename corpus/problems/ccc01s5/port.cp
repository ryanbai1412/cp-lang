// Source: CCC 2001 S5 (Post's Correspondence Problem), https://dmoj.ca/problem/ccc01s5
// Problem: words a_1..a_n and b_1..b_n of at most 20 letters, m * n <= 40. Find indices
//   i_1..i_k, 0 < k < m, repeats allowed, with a_i1 ... a_ik = b_i1 ... b_ik. Print k and the
//   list, one number to a line, or "No solution.".
// Solution: depth-first over lists, keeping only the part by which one row is ahead of the
//   other; a pick whose rows disagree is dropped at once, and lists stop at m - 1 words.
//   Exponential in the worst case, but m * n <= 40 keeps it small.
// original: corpus/submissions/5025769.cpp
let (m, n): (1..=40, 1..=40) = read()
assert m * n <= 40
domain Pair = 1..=n
let a: [Pair]Str = read()
let b: [Pair]Str = read()

var picked: List<Pair> = []
/// Extend the list given what the top row (or, with `top_ahead` false, the bottom row) has spare.
fn search(ahead: Str, top_ahead: Bool) -> Bool:
    if not picked.is_empty and ahead.is_empty: return true
    if picked.len == m - 1: return false
    for p in Pair:
        let (long, short) = if top_ahead: (ahead + a[p], b[p]) else: (ahead + b[p], a[p])
        picked.push(p)
        if long.starts_with(short):
            if search(long[short.len..long.len], top_ahead): return true
        elif short.starts_with(long):
            if search(short[long.len..short.len], not top_ahead): return true
        picked.pop()
    false

if search("", true):
    print(picked.len)
    for p in picked: print(p)
else: print("No solution.")
