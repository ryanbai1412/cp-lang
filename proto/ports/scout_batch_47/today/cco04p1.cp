// CCO '04 P1 — Scribble (today form for proto/cpc)
// original: corpus/submissions/5430016.cpp
let m: 0..=26 = read()
domain Alpha = 0..256
var score: [Alpha]Int = 0
var avail: [Alpha]Int = 0

for _ in 0..m:
    let ch: Char = read()
    let val: Int = read()
    let lim: Int = read()
    let code = ch - 'a' + 97
    score[Alpha(code)] = val
    avail[Alpha(code)] = lim

let n: 0..=100_000 = read()

fn eval_word() -> Int:
    let w: [domain W]Char = read()
    var used: [Alpha]Int = 0
    for i in W:
        let code = w[i] - 'a' + 97
        used[Alpha(code)] += 1
    var total = 0
    for i in W:
        let code = w[i] - 'a' + 97
        if used[Alpha(code)] > avail[Alpha(code)]: return 0
        total += score[Alpha(code)]
    total

var best = 0
for _ in 0..n:
    let cur = eval_word()
    if cur > best: best = cur
print(best)
