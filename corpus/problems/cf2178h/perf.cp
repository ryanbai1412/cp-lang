// Codeforces 2178 H: Create or Duplicate, tuned.
// port.cp's search, written for its state: a state is `8 * r + started`, one
// flat distance row, and the six moves each keep a pointer into the one
// settling order, as orig.cpp does, instead of `fixed_cost_paths` over
// `(R, Subset<T>)` states and `Move` values.

@!multitest
let x: [0..3]1..=500_000 = read()
let (m, k): (2..=500_000, 1..=500_000) = read()

let price = [x[0], x[1], x[2], k, 2 * k, 3 * k]
let started = [0, 1, 1, 2, 1, 2, 2, 3]
var dist: [0..8 * m]?(0..2_100_000_000) = None
var order: List<0..8 * m> = []
order.reserve(8 * m)
dist[0] = 0
order.push(0)
var at = [0, 0, 0, 0, 0, 0]
var total = 0
while true:
    var best: ?(d: Int, z: 0..6, t: 0..8 * m) = None
    for z in 0..6:
        while at[z] < order.len:
            let s = order[at[z]]
            let set = s % 8
            if z >= 3 and started[set] != z - 2:
                at[z] += 1
                continue
            var r = if z < 3: s / 8 + x[z] else: s / 8 * 2
            if r >= m: r -= m
            let t = 8 * r + (if z < 3: set | 1 << z else: set)
            if dist[t] != None:
                at[z] += 1
                continue
            let d = dist[s]! + price[z]
            if best.map(b => d < b.d) ?? true: best = (d:, z:, t:)
            break
    let (d, z, t) = best!
    if t == 7:
        total = d
        break
    dist[t] = d
    order.push(t)
    at[z] += 1
print(total - x.sum())
