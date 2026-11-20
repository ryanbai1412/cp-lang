// DMOPC '20 Contest 4 P4 (DMOJ: dmopc20c4p4) - cheapest prefix rearrangement
// Ideal v2 port. original: corpus/submissions/3485716.cpp
//
// For every prefix print the minimum cost to make it valid, where element i
// asks for a[i] earlier elements kept in place. `have` counts kept elements;
// an element that asks for fewer than `have` is set aside at a cost and can be
// pulled back later (latest first) when a larger request needs it.
let n: 1..=1e6 = read()
let a: [0..n]Int = read()

var (have, cost) = (0, 0)
var aside: List<Int> = []
var out: List<Int> = []
for (i, need) in a.items:
    while have < need:
        cost += i - aside.pop()!
        have += 1
    if need == have:
        have += 1
    else:
        cost += have - need - 1
        aside.push(i)
    out.push(cost)
print(" ".join("{c}" for c in out))
