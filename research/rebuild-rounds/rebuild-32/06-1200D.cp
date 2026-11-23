// cf1200d White Lines: each non-white row r is whitened by the clicks (i, j) with
// i in r - k + 1..=r and j in last_B - k + 1..=first_B; columns likewise. Add each
// such rectangle to a 2D difference array and take the best click.
let n: 1..=2000 = read()
let k: 1..=n = read()
domain R = 1..=n
domain C = 1..=n
let g = read<[R, C]Char>()

let last_click = n - k + 1
domain D = 1..=n + 1
var diff: [D, D]Int = 0
var base = 0
var rects = List<(Int, Int, Int, Int)>()

for r in R:
    // found late: g[r].keys are 0-based Ints, not C, so read cells by (r, c)
    if let first = C.find(c => g[r, c] == 'B'):
        let last = C.rev.find(c => g[r, c] == 'B')!
        rects.push((Int(r) - k + 1, Int(r), Int(last) - k + 1, Int(first)))
    else: base += 1

for c in C:
    if let first = R.find(r => g[r, c] == 'B'):
        let last = R.rev.find(r => g[r, c] == 'B')!
        rects.push((Int(last) - k + 1, Int(first), Int(c) - k + 1, Int(c)))
    else: base += 1

for (i1, i2, j1, j2) in rects:
    let lo_i = max(i1, 1)
    let hi_i = min(i2, last_click)
    let lo_j = max(j1, 1)
    let hi_j = min(j2, last_click)
    if lo_i <= hi_i and lo_j <= hi_j:
        diff[D(lo_i), D(lo_j)] += 1
        diff[D(lo_i), D(hi_j + 1)] -= 1
        diff[D(hi_i + 1), D(lo_j)] -= 1
        diff[D(hi_i + 1), D(hi_j + 1)] += 1

for i in D:
    for j in D:
        if let pi = i.prev: diff[i, j] += diff[pi, j]
        if let pj = j.prev: diff[i, j] += diff[i, pj]
        if let pi = i.prev and let pj = j.prev: diff[i, j] -= diff[pi, pj]

domain Click = 1..=last_click
print(base + Click.map(i => Click.map(j => diff[D(Int(i)), D(Int(j))]).max()!).max()!)
