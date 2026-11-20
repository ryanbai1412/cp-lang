// CCO '04 P1 — Scribble (DMOJ: cco04p1)
// original: corpus/submissions/5430016.cpp
// CP Lang v2 (locked spec, docs/one-page-cheat-sheet.md @ 3723dca)
//
// Given m letter types (char, score, available count) and n candidate words,
// find the maximum score among words buildable from the available letter tiles.
let m: 0..=26 = read()
var score: [0..256]Int = 0
var avail: [0..256]Int = 0
for _ in 0..m:
    let (ch, val, lim): (Char, Int, Int) = read()
    let code = ch - '\0'
    score[code] = val
    avail[code] = lim

let n: 0..=1e5 = read()
var best = 0
for _ in 0..n:
    let word: Str = read()
    var used: [0..256]Int = 0
    for ch in word: used[ch - '\0'] += 1
    if all(used[ch - '\0'] <= avail[ch - '\0'] for ch in word):
        best max= sum(score[ch - '\0'] for ch in word)

print(best)
