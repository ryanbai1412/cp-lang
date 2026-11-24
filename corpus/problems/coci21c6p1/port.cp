// Source: COCI 2021/2022 Round 6 P1 (Med), https://dmoj.ca/problem/coci21c6p1
// limits: 1000 ms, 512 MB
// Problem: n <= 500 beekeepers with distinct names and five round scores in 0..=500; the sixth
//   round adds 0..=500 more. Ranking is by total, ties to the smaller name. For each, print the
//   best and the worst possible final place.
// Solution: best place: they get 500 and everyone else 0; worst: they get 0 and everyone else
//   500. Each place is 1 + the number ranked ahead, counted directly: O(n^2).
// original: corpus/submissions/4529987.cpp
// Bounds: n: 1..=500; rounds: 0..=500

let n: 1..=500 = read()
domain Keeper = 0..n
var name: [Keeper]Str = ""
var total: [Keeper]Int = 0
for k in Keeper:
    name[k] = read()
    let rounds: [5]0..=500 = read()
    total[k] = rounds.sum()

// ahead ranks first: the higher total, then the smaller name
fn place(me: Keeper, mine: Int, theirs: Int) -> Int =
    1 + Keeper.count(o => o != me and (total[o] + theirs, desc(name[o])) > (total[me] + mine, desc(name[me])))

for k in Keeper: print(place(k, 500, 0), place(k, 0, 500))
