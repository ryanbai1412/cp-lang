// Source: CEOI '19 Practice P2 (Separator), https://dmoj.ca/problem/ceoi19pp2
// limits: 650 ms, 512 MB
// Problem: append n <= 1e6 distinct values to an initially empty sequence, online: the i-th is
//   a_i = (b_i + s_{i-1}) mod 1e9, given b_i < 1e9. After each append print s_i, the number of
//   separators: indices j with every earlier element smaller and every later one larger.
// Solution: a separator is a prefix maximum that no later element undercuts. Keep the current
//   separators on an increasing stack: a new prefix maximum is pushed, any other element pops
//   every kept value larger than it. Each value enters and leaves once, so the whole run is
//   O(n); s_i is the stack's size.
// Bounds: q: 1..=1e6; bs: 0..1e9
// original: corpus/submissions/5343197.cpp
let q: 1..=1e6 = read()
let bs: [q]0..1e9 = read()
var kept: List<Int> = []
var high: ?Int = None
for b in bs:
    let x = (b + kept.len) % 1e9
    if high == None or x > high!:
        high = x
        kept.push(x)
    else:
        kept.pop_while(t => t > x)
    print(kept.len)
