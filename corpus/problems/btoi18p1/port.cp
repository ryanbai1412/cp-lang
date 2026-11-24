// Source: Baltic OI 2018 Love Polygon, https://dmoj.ca/problem/btoi18p1
// limits: 1000 ms, 1024 MB
// Problem: n <= 1e5 people (names of at most 10 letters) each love one person, maybe themselves.
//   Print the fewest love arrows to redirect so that everyone is in a pair of two people loving
//   each other, or -1 if n is odd.
// Solution: the answer is n minus the arrows kept. Mutual pairs keep both. Elsewhere a kept arrow
//   is a matched edge of the functional graph: match greedily from the leaves inward (peeling
//   in-degree 0), then along what is left of each cycle, starting after an already matched node
//   if the cycle has one, else anywhere (len / 2). O(n) after interning names.
// original: corpus/submissions/4456169.cpp
// Bounds: n: 1..=1e5
use std.marks.{Interner}

let n: 1..=1e5 = read()
domain P = 0..n
var names = Interner<Str>()
var loves = [P] p => p
repeat n:
    let (from, to): (Str, Str) = read()
    loves[P(names.id(from))] = P(names.id(to))

// the most arrows that can stay, each person then in a loving pair
fn kept() -> Int:
    var paired: [P]Bool = false
    var total = 0
    fn pair(p: P, o: P):  // p's arrow stays and o's turns back to p
        (paired[p], paired[o]) = (true, true)
        total += 1
    for p in P:
        let o = loves[p]
        if p < o and loves[o] == p:
            pair(p, o)
            total += 1  // o's arrow already points back

    // Peel the people nobody loves, matching each with whom they love while both are free.
    var fans: [P]0..=n = 0
    for p in P: fans[loves[p]] += 1
    var free = [p for p in P if fans[p] == 0]
    while let p = free.pop():
        let o = loves[p]
        if not paired[p] and not paired[o] and o != p: pair(p, o)
        fans[o] -= 1
        if fans[o] == 0: free.push(o)

    // Whole cycles are left. Match along each from just after a matched person, if any.
    for start in P:
        if fans[start] == 0: continue
        var ring = [start]
        while loves[ring.last!] != start: ring.push(loves[ring.last!])
        for p in ring: fans[p] = 0
        if let i = (0..ring.len).find(i => paired[ring[i]]):
            for (p, o) in ring.rotated_left(i + 1).adjacent:
                if not paired[p] and not paired[o]: pair(p, o)
        else: total += ring.len / 2
    total

print(if n % 2 == 1: -1 else: n - kept())
