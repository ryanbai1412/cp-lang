// Baltic OI 2018 Love Polygon: n people, each loves one person (maybe
// themselves). Redirect as few loves as possible so that everybody is in a
// pair loving each other; print that number, or -1 if n is odd.
//
// A final pair costs 2 minus the loves already pointing inside it, so the
// goal is to keep as many loves as possible. Pairs that already love each
// other stay. Everything else is a matching on the love arrows: take them
// from the leaves inward, then walk what is left of each cycle.
// original: corpus/submissions/4456169.cpp
use std.seq.{get_or_insert_with}

let n: Int = read()
domain P = 0..n
var ids = Map<Str, P>()

fn id(name: Str) -> P:
    let next_id = ids.len
    ids.get_or_insert_with(name, () => P(next_id))

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
