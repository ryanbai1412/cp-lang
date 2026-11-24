// Source: CCO 2004 Stage 2 Day 1 P1 (Scribble), https://dmoj.ca/problem/cco04p1
// Problem: 7 tiles as k letters, each with a score <= 26 and a count. Of N <= 1e5 words, print
//   the best score (sum of its letters' scores) of one spelled from the tiles, or 0 if none.
// Solution: count each word's letters; it is playable iff no count exceeds that letter's
//   tiles, and then it scores the sum of its letters. O(total length).
// original: corpus/submissions/5430016.cpp
// Bounds: k: 0..=26; tiles.score: 0..=26; tiles.count: 0..=7; n: 0..=1e5
domain Letter = 'a'..='z'
let k: 0..=26 = read()
let tiles: [k](letter: Char, score: 0..=26, count: 0..=7) = read()
var score: [Letter]Int = 0
var have: [Letter]Int = 0
for t in tiles:
    score[Letter(t.letter)] = t.score
    have[Letter(t.letter)] = t.count

let n: 0..=1e5 = read()
var best = 0
for _ in 0..n:
    let w: Str = read()
    let word = [Letter(c) for c in w]
    var used: [Letter]Int = 0
    for c in word: used[c] += 1
    if all(used[c] <= have[c] for c in Letter): best max= sum(score[c] for c in word)
print(best)
