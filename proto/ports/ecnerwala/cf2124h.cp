@!multitest
let n0: Int = read()
let n = n0 + 1
var a: [0..n]Int = 0
for i in 1..n:
    a[i] = read()
var max_len: [0..(n * (n + 1) / 2)]Int = 0
fn tidx(i: Int, j: Int) -> Int = j * (j + 1) / 2 + i
for i in (0..n).rev:
    let ai = a[i]
    var first_vis: [0..(n + 1)]Int = -1
    var cur_best = ai - 1
    for j in i..n:
        let aj = a[j]
        if aj < ai: continue
        if aj == ai:
            cur_best += 1
            max_len[tidx(i, j)] = cur_best
        elif aj <= cur_best + 1:
            if first_vis[aj] == -1: first_vis[aj] = j
            cur_best max= max_len[tidx(first_vis[aj], j)]
    if i == 0: print(cur_best)
