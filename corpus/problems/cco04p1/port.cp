// Source: CCO 2004 Stage 2 Day 1 P1 (Scribble), https://dmoj.ca/problem/cco04p1
// Problem: 7 tiles as k letters, each with a score <= 26 and a count. Of N <= 1e5 words, print
//   the best score (sum of its letters' scores) of one spelled from the tiles, or 0 if none.
// Solution: count each word's letters; it is playable iff no count exceeds that letter's
//   tiles, and then it scores the sum of its letters. O(total length).
// original: corpus/submissions/5430016.cpp
// Bounds: m: 0..=26; n: 0..=100_000
let m: 0..=26 = read()
domain Alpha = 0..256
var score: [Alpha]Int = 0
var avail: [Alpha]Int = 0

for _ in 0..m:
    let ch: Char = read()
    let val: Int = read()
    let lim: Int = read()
    let code = ch - 'a' + 97
    score[Alpha(code)] = val
    avail[Alpha(code)] = lim

let n: 0..=100_000 = read()

fn eval_word() -> Int:
    let w: [domain W]Char = read()
    var used: [Alpha]Int = 0
    for i in W:
        let code = w[i] - 'a' + 97
        used[Alpha(code)] += 1
    var total = 0
    for i in W:
        let code = w[i] - 'a' + 97
        if used[Alpha(code)] > avail[Alpha(code)]: return 0
        total += score[Alpha(code)]
    total

var best = 0
for _ in 0..n:
    let cur = eval_word()
    best max= cur
print(best)
