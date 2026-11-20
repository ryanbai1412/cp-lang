// COCI 2021/22 #6 Med: n contestants with five round scores each; one round
// worth up to 500 points is left. Ranking is by total, higher first, ties
// by name. For each contestant print the best and the worst final place.
//
// Best: they gain 500 and nobody else scores. Worst: everyone else gains 500.
// original: corpus/submissions/4529987.cpp
let n: 1..=500 = read()
let rows: [0..n](name: Str, rounds: [0..5]0..=500) = read()
let total = [0..n] i => sum(rows[i].rounds)

// 1 + how many others finish ahead when i gains `mine` and the others `theirs`
fn place(i: Int, mine: Int, theirs: Int) -> Int:
    let me = (-(total[i] + mine), rows[i].name)
    1 + count(j for j in 0..n if j != i and (-(total[j] + theirs), rows[j].name) < me)

for i in 0..n: print(place(i, 500, 0), place(i, 0, 500))
