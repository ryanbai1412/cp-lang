// Runnable port (proto/cp.py lib-run) of statement-first 996warmup8p3.cp
// original: corpus/submissions/5324670.cpp
let L = 1000000001
let (n, kl0, kr0): (Int, Int, Int) = read()
var pos: List<Int> = []
var dir: List<Int> = []
for _ in 0..n:
    let p: Int = read()
    pos.push(p)
for _ in 0..n:
    let d: Int = read()
    dir.push(d)

// State after x seconds: hits on the left wall, hits on the right wall, and
// each ball as (position, heading right?).
var hl = 0
var hr = 0
var at: List<Int> = []
var right: List<Bool> = []
fn simulate(x: Int):
    let m = x % (2 * L)
    hl = n * (x / (2 * L))
    hr = hl
    at = []
    right = []
    for i in 0..n:
        // Mirror right-movers so every ball heads for "its" wall at distance p.
        let flip = dir[i] != 0
        let p = if flip: L - pos[i] else: pos[i]
        var (np, turned) = (p - m, false)
        if m >= p + L:
            hl += 1
            hr += 1
            np = 2 * L - m + p
        elif m >= p:
            if flip: hr += 1
            else: hl += 1
            np = m - p
            turned = true
        at.push(if flip: L - np else: np)
        right.push(turned != flip)

var (lo, hi) = (0, 1000000000 * 2 * L / n + 2)
while lo < hi:
    let mid = (lo + hi) / 2
    simulate(mid)
    if hl >= kl0 or hr >= kr0: hi = mid
    else: lo = mid + 1
let t1 = lo
simulate(t1)

if hl >= kl0 and hr >= kr0:
    print(t1 + L)
else:
    // One wall is done; orient so it is the left one and count what the right still needs.
    let swap_sides = hr >= kr0
    let need = if swap_sides: kl0 - hl else: kr0 - hr
    var movers: List<Int> = []
    for i in 0..n:
        let p = if swap_sides: L - at[i] else: at[i]
        if right[i] != swap_sides: movers.push(p)
    sort(mut movers, by: p => -p)
    if need < movers.len: print(t1 + 2 * L - movers[need - 1])
    else: print(t1 + 2 * L)
