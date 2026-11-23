// Source: COCI 2021/2022 Round 6 P1 (Med), https://dmoj.ca/problem/coci21c6p1
// Problem: n <= 500 beekeepers with distinct names and five round scores in 0..=500; the sixth
//   round adds 0..=500 more. Ranking is by total, ties to the smaller name. For each, print the
//   best and the worst possible final place.
// Solution: best place: they get 500 and everyone else 0; worst: they get 0 and everyone else
//   500. Each place is 1 + the number ranked ahead, counted directly: O(n^2).
// original: corpus/submissions/4529987.cpp
let n: Int = read()
var name: List<Str> = []
var total: List<Int> = []
for _ in 0..n:
    let who: Str = read()
    let rounds: [5]Int = read()
    name.push(who)
    total.push(rounds.sum())

fn place(i: Int, mine: Int, theirs: Int) -> Int:
    let me = total[i] + mine
    var ahead = 0
    for j in 0..n:
        let them = total[j] + theirs
        if j != i and (them > me or (them == me and name[j] < name[i])): ahead += 1
    1 + ahead

for i in 0..n: print(place(i, 500, 0), place(i, 0, 500))
