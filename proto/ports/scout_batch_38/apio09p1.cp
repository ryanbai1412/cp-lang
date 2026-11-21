// APIO '09 P1 (Digging for Oil): pick 3 non-overlapping k x k sub-squares
// of an n x m grid to maximize their total sum. Every optimal packing splits
// along a horizontal or vertical line (2 on one side, 1 on the other), so
// four orientations cover all configurations.
// original: corpus/submissions/5536569.cpp
let (n, m, k): (1..=1500, 1..=1500, 1..=1500) = read()
domain Row = 0..n
domain Col = 0..m
let g: [Row, Col]Int = read()

fn max_below<R, C>(g: [R, C]Int, k: Int) -> [R.cuts]?Int:
    let pre = prefix(g)
    var down: [R.cuts]?Int = none
    for rw in R.windows(k).rev:
        let b = rw.first!.l                       // FRICTION 38.2: non-empty window .first is ?R
        if let nxt = rw.first! + 1: down[b] = down[nxt.l]
        for cw in C.windows(k):
            down[b] max= pre.sum(rw, cw)
    down

fn best_oriented<R, C>(g: [R, C]Int, k: Int) -> Int:
    let pre = prefix(g)
    let down = max_below(g, k)
    // FRICTION 38.1: g.flipped is axis-ambiguous on 2D grids (row-flip vs col-flip)
    var up: [R.cuts]?Int = none
    for rw in R.windows(k):
        let b = rw.last!.r
        if let prv = rw.last! - 1: up[b] = up[prv.r]
        for cw in C.windows(k):
            up[b] max= pre.sum(rw, cw)

    var pref: [C.cuts]?Int = none
    var suff: [C.cuts]?Int = none
    var two_above: ?Int = none
    var ans = 0
    for rw in R.windows(k):
        var row_max: ?Int = none
        for cw in C.windows(k):
            let s = pre.sum(rw, cw)
            pref[cw.last!.r] max= s
            suff[cw.first!.l] max= s
            row_max max= s
        // FRICTION 38.2: scan() is forward-only and allocating; no in-place or reverse scan
        for c in C: pref[c.r] max= pref[c.l]
        for c in C.rev: suff[c.l] max= suff[c.r]
        if let u = up[rw.first!.l]:
            two_above max= row_max! + u
        // FRICTION 38.4: `if let l = ... and let r = ...` is in decisions.md but omitted from cheat sheet
        for b in C.cuts:
            if let l = pref[b] and let r = suff[b]:
                two_above max= l + r
        if let t = two_above and let d = down[rw.last!.r]:
            ans max= t + d
    ans

print(max([
    best_oriented(g, k),
    best_oriented(g.flipped, k),                  // FRICTION 38.1: assumes vertical (row) flip
    best_oriented(g.transposed, k),
    best_oriented(g.transposed.flipped, k),
])!)
