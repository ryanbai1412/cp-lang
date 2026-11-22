// Runnable port (proto/cp.py lib-run) of statement-first rpeng3.cp
// original: corpus/submissions/4811952.cpp
let (n, up, down, home): (Int, Int, Int, Int) = read()
let NEG = -1000000000000000000

var fairs: List<(t: Int, at: Int, gain: Int)> = []
var top = home
for _ in 0..n:
    let (t, at, gain): (Int, Int, Int) = read()
    fairs.push((t:, at:, gain:))
    top max= at
fairs.push((t: 1 << 19, at: home, gain: 0))     // the trip ends back home
fairs.sort()

// from_low[p] = best profit ending at p, plus p * down: a later stop x >= p
// subtracts x * down. from_high mirrors it for stops x <= p. The interpreter
// version scans these arrays; the ideal port keeps them in two max SegTrees.
var from_low = List.filled(top + 1, NEG)
var from_high = List.filled(top + 1, NEG)
from_low[home] = home * down
from_high[home] = -home * up

fn arrive(x: Int) -> Int:
    var best = NEG
    for p in 0..x + 1: best max= from_low[p] - x * down
    for p in x..top + 1: best max= from_high[p] + x * up
    return best

var i = 0
while i < fairs.len:
    var j = i
    while j < fairs.len and fairs[j].t == fairs[i].t: j += 1
    // Fairs i..j share a day: visit a contiguous stretch, entering anywhere,
    // sweeping to one end and optionally back.
    let m = j - i
    var at: List<Int> = []
    var pre = [0]
    var entry: List<Int> = []
    for q in i..j:
        at.push(fairs[q].at)
        pre.push(pre.last! + fairs[q].gain)
        entry.push(arrive(fairs[q].at))
    var opt = List.filled(m, NEG)
    var val = List.filled(m, NEG)
    // Enter at q, go up to the far end, come back down to the exit.
    var run = NEG
    for q in (0..m).rev:
        run max= -(up + down) * at[q] + pre[q + 1]
        val[q] = run + (up + down) * at[q] - pre[q] + entry[q]
    run = NEG
    for q in (0..m).rev:
        run max= -up * at[q] + pre[q] + val[q]
        opt[q] max= run + up * at[q] - pre[q]
    // Enter at q, go down to the near end, come back up to the exit.
    run = NEG
    for q in 0..m:
        run max= (up + down) * at[q] - pre[q]
        val[q] = run - (up + down) * at[q] + pre[q + 1] + entry[q]
    run = NEG
    for q in 0..m:
        run max= down * at[q] - pre[q + 1] + val[q]
        opt[q] max= run - down * at[q] + pre[q + 1]
    for q in 0..m:
        from_low[at[q]] = opt[q] + at[q] * down
        from_high[at[q]] = opt[q] - at[q] * up
    i = j
print(from_low[home] - home * down)
