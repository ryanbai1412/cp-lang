// APIO 2007 Backup: n offices stand along a street. Pair up 2k of them with
// k cables, each office on at most one cable, using the least cable in total.
// original: corpus/submissions/5532752.cpp
let n: 2..=1e5 = read()
let k: 1..=n / 2 = read()
domain Office = 1..=n
domain Gap = 1..n                           // gap g lies between offices g and g + 1

fn street() -> [Office]0..=1e9:
    var at: [Office]0..=1e9 = 0
    var here = 0
    for o in Office:
        here += rand(1, 1e4)
        at[o] = here
    at

let at: [Office]0..=1e9 = read(gen: street)

// A best plan only joins neighbours, so it picks k gaps, no two adjacent.
// Take the cheapest gap g. If that was wrong, the plan uses both gaps beside
// g instead, so g stays in the row as one gap that costs left + right - g
// ("undo g") and swallows its neighbours. A gap at the end of the row has
// nothing to trade for: it just leaves with its neighbour.
var cost: [Gap]Int = [Gap] g => at[Office(g.label + 1)] - at[Office(g.label)]
var left: [Gap]?Gap = [Gap] g => g - 1
var right: [Gap]?Gap = [Gap] g => g + 1
var gone: [Gap]Bool = false

fn remove(g: Gap):
    gone[g] = true
    if let l = left[g]: right[l] = right[g]
    if let r = right[g]: left[r] = left[g]

var cheapest = MinHeap<(Int, Gap)>()
for g in Gap: cheapest.push((cost[g], g))
var cable = 0
var laid = 0
while laid < k:
    if let (c, g) = cheapest.pop():
        if gone[g] or c != cost[g]: continue
        cable += c
        laid += 1
        let (l, r) = (left[g], right[g])
        if let a = l: remove(a)
        if let b = r: remove(b)
        if l != none and r != none:
            cost[g] = cost[l ?? g] + cost[r ?? g] - c       // WISH: if let (a, b) = (l, r)
            cheapest.push((cost[g], g))
        else:
            remove(g)
print(cable)
