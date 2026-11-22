// CCC 2001 S5 Post's Correspondence Problem: m, n, then words a1..an and
// b1..bn (n <= 40, m <= 40, each word at most 20 letters). Find k < m and a
// list i1..ik, repeats allowed, with a[i1] + ... + a[ik] equal to
// b[i1] + ... + b[ik]. Print k and then the list, one number to a line, or
// "No solution." if there is none.
//
// Depth-first over lists, keeping only the part by which one row is ahead
// of the other; a list whose rows already disagree is dropped at once.
// original: corpus/submissions/5025769.cpp
let (m, n): (Int, Int) = read()
domain Pair = 1..=n
let a: [Pair]Str = read()
let b: [Pair]Str = read()

fn starts_with(s: Str, head: Str) -> Bool = head.len <= s.len and s[0..head.len] == head

var picked: List<Pair> = []
/// Extend the list given what the top row (or, with `top_ahead` false, the bottom row) has spare.
fn search(ahead: Str, top_ahead: Bool) -> Bool:
    if not picked.is_empty and ahead.is_empty: return true
    if picked.len == m - 1: return false
    for p in Pair:
        let (long, short) = if top_ahead: (ahead + a[p], b[p]) else: (ahead + b[p], a[p])
        picked.push(p)
        if starts_with(long, short):
            if search(long[short.len..long.len], top_ahead): return true
        elif starts_with(short, long):
            if search(short[long.len..short.len], not top_ahead): return true
        picked.pop()
    false

if search("", true):
    print(picked.len)
    for p in picked: print(p)
else: print("No solution.")
