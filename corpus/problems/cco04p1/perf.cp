// Tuned: one pass per word counts, checks and scores its letters in a single tally reused
// across words, where the port builds the word's letters and a fresh tally for each word.
//
// Source: CCO 2004 Stage 2 Day 1 P1 (Scribble), https://dmoj.ca/problem/cco04p1
// limits: 1000 ms, 32 MB
// Problem: 7 tiles as k letters, each with a score <= 26 and a count. Of N <= 1e5 words, print
//   the best score (sum of its letters' scores) of one spelled from the tiles, or 0 if none.
// Solution: count each word's letters; it is playable iff no count exceeds that letter's
//   tiles, and then it scores the sum of its letters. O(total length).
// original: corpus/submissions/5430016.cpp
// Bounds: k: 1..=7; tiles.score: 0..=26; tiles.count: 0..=7; n: 0..=1e5
domain Letter = 'a'..='z'
let k: 1..=7 = read()
let tiles: [k](letter: Char, score: 0..=26, count: 0..=7) = read()
var score: [Letter]Int = 0
var have: [Letter]Int = 0
for t in tiles:
    score[Letter(t.letter)] = t.score
    have[Letter(t.letter)] = t.count

let n: 0..=1e5 = read()
var best = 0
var used: [Letter]Int = 0
for _ in 0..n:
    let w: Str = read()
    var total = 0
    var fits = true
    for c in w:
        let l = Letter(c)
        used[l] += 1
        if used[l] > have[l]: fits = false
        total += score[l]
    if fits: best max= total
    for c in w: used[Letter(c)] = 0
print(best)
