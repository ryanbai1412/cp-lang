// SAC '22 CC2 P3 (today/ runnable subset) — original: corpus/submissions/4121049.cpp

let k: 1..=19 = read()
let r: 0..=1_000_000_000 = read()
var n = 1
for _ in 0..k:
    n *= 2

// each of the n - 1 lines is a fork at u: take a for w, or take b for free
var to: List<Int> = []
var gain: List<Int> = []
var out: [0..2 * n]List<Int> = []
for _ in 0..n - 1:
    let u: 1..=1_048_576 = read()
    let a: 1..=1_048_576 = read()
    let b: 1..=1_048_576 = read()
    let w: -1_000_000_000..=1_000_000_000 = read()
    out[u - 1].push(to.len)
    to.push(a - 1)
    gain.push(w)
    out[u - 1].push(to.len)
    to.push(b - 1)
    gain.push(0)

var order: List<Int> = []
order.push(0)
var head = 0
while head < order.len:
    let u = order[head]
    head += 1
    for j in 0..out[u].len:
        order.push(to[out[u][j]])

var best: [0..2 * n]Int = 0
for i in 0..order.len:
    let u = order[order.len - 1 - i]
    for j in 0..out[u].len:
        let e = out[u][j]
        let cand = gain[e] + best[to[e]]
        if j == 0 or cand > best[u]:
            best[u] = cand
print(r + best[0])
