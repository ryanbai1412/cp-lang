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
use std.marks.{Interner}

let n: Int = read()
domain P = 0..n
var ids = Interner<Str>()

fn id(name: Str) -> P:
    P(ids.id(name))

var loves = [P] p => p
for _ in P:
    let (s, t): (Str, Str) = read()
    loves[id(s)] = id(t)

fn kept() -> Int:
    var paired: [P]Bool = false
    var total = 0
    for p in P:
        let o = loves[p]
        if o != p and loves[o] == p:
            paired[p] = true
            total += 1  // counted from both sides: 2 per pair

    var fans: [P]Int = 0
    for p in P: fans[loves[p]] += 1
    var free = [p for p in P if fans[p] == 0]
    while let p = free.pop():
        let o = loves[p]
        if not paired[p] and not paired[o] and o != p:
            paired[p] = true
            paired[o] = true
            total += 1
        fans[o] -= 1
        if fans[o] == 0: free.push(o)

    // Only whole cycles still have fans. Pair along each from a broken
    // spot if there is one, else from anywhere.
    for start in P:
        if fans[start] == 0 or not paired[start]: continue
        var p = loves[start]
        while p != start:
            fans[p] = 0
            let o = loves[p]
            if not paired[p] and not paired[o] and o != start:
                paired[p] = true
                paired[o] = true
                total += 1
            p = o
        fans[start] = 0

    for start in P:
        if fans[start] == 0: continue
        var len = 1
        var p = loves[start]
        while p != start:
            fans[p] = 0
            len += 1
            p = loves[p]
        fans[start] = 0
        total += len / 2
    total

print(if n % 2 == 1: -1 else: n - kept())
