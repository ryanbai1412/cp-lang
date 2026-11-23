// cf1077f2 Pictures with Kittens (hard): exactly x reposts, every k-run has one.
// best[j][i]: j reposts among pictures 1..=i with picture i reposted (i = 0 is the start).
// best[j][i] = a[i] + max(best[j - 1][i - k..i]): a sliding-window max per layer.
let n: 1..=5000 = read()
let k: 1..=n = read()
let x: 1..=n = read()
let a = read<[n]1..=1e9>()

let NEG = -(1 << 60)
var prev: [0..=n]Int = NEG
prev[0] = 0
for _ in 1..=x:
    var cur: [0..=n]Int = NEG
    var w = Window(max)
    for i in 1..=n:
        w.push(prev[i - 1])
        if w.len > k: w.pop()
        if w.all! > NEG: cur[i] = w.all! + a[i - 1]
    prev = cur

let best = max(prev[i] for i in n - k + 1..=n)!
print(if best > NEG: best else: -1)
