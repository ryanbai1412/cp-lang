// Source: IOI 2014 Day 2 Friend, https://dmoj.ca/problem/ioi14p5
// Problem: a network of n <= 1e5 people grows from person 0: person i joins via a host h < i as
//   a friend of the host only (protocol 0), of the host's friends only (1), or of both (2).
//   Print the largest total confidence (each <= 1e4) of a set with no two friends.
// Solution: undo the growth from the last person back, folding each person into its host; a
//   person then stands for a group with two totals, best with that person picked and without.
//   Protocol 0 merges like a tree edge (not both), 1 like twins with the same friends (both
//   allowed), 2 like adjacent twins (at most one). O(n).
// input: n, the n confidences, then `host protocol` for persons 1..n-1
// original: corpus/submissions/6567598.cpp
// Bounds: n: 2..=1e5; conf: 1..=1e4; joins.host: 0..=n - 1; joins.how: 0..=2
let n: 2..=1e5 = read()
domain P = 0..n
domain Joiner = 1..n  // the people who joined, 1..n-1
let conf: [P]1..=1e4 = read()
let joins: [Joiner](host: P, how: 0..=2) = read()

var pick: [P]Int = [P] p => conf[p]  // best of the group with its person picked
var skip: [P]Int = 0  // ... and with it left out
for j in Joiner.rev:
    let (u, (h, how)) = (P(j.label), joins[j])
    let either = max(pick[u], skip[u])
    if how == 0:  // friends with the host: not both
        (pick[h], skip[h]) = (pick[h] + skip[u], skip[h] + either)
    elif how == 1:  // same friends as the host: both or either
        (pick[h], skip[h]) = (max(pick[h] + either, skip[h] + pick[u]), skip[h] + skip[u])
    else:  // both rules: exactly one of the two, or neither
        (pick[h], skip[h]) = (max(pick[h] + skip[u], skip[h] + pick[u]), skip[h] + skip[u])
print(max(pick[P.first!], skip[P.first!]))
