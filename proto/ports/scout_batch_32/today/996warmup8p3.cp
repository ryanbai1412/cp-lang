// 996 Warmup 8 P3 (CP Lang 0.2.2): `bsearch_first` replaces manual `while lo < hi:`
// binary search and domain-indexed `read()` replaces manual input loops.
// original: corpus/submissions/5324670.cpp

const L = 1_000_000_001
let (n, kl0, kr0): (Int, Int, Int) = read()
domain Ball = 0..n
let pos: [Ball]Int = read()
let dir: [Ball]Int = read()

var hl = 0
var hr = 0
var at: [Ball]Int = 0
var right: [Ball]Bool = false

fn simulate(x: Int):
    let m = x % (2 * L)
    hl = n * (x / (2 * L))
    hr = hl
    for i in Ball:
        let flip = dir[i] != 0
        let p = if flip: L - pos[i] else: pos[i]
        var (np, turned) = (p - m, false)
        if m >= p + L:
            hl += 1
            hr += 1
            np = 2 * L - m + p
        elif m >= p:
            if flip: hr += 1 else: hl += 1
            np = m - p
            turned = true
        at[i] = if flip: L - np else: np
        right[i] = turned != flip

fn hits_enough(mid: Int) -> Bool:
    simulate(mid)
    hl >= kl0 or hr >= kr0

let hi_bound = 1_000_000_000 * 2 * L / n + 2
let t1 = bsearch_first(0..=hi_bound, hits_enough)!
simulate(t1)

if hl >= kl0 and hr >= kr0:
    print(t1 + L)
else:
    let swap_sides = hr >= kr0
    let need = if swap_sides: kl0 - hl else: kr0 - hr
    var movers = [if swap_sides: L - at[i] else: at[i] for i in Ball if right[i] != swap_sides]
    movers.sort(by: p => -p)
    if need < movers.len: print(t1 + 2 * L - movers[need - 1])
    else: print(t1 + 2 * L)
