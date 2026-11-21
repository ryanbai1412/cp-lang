@!multitest
let n = read::<Int>() + 1
let a = [0..n] i => if i == 0: 0 else: read::<Int>()
var max_len: [0..n * (n + 1) / 2]Int = 0
fn tidx(i: Int, j: Int) -> Int = j * (j + 1) / 2 + i

for i in (0..n).rev:
    let ai = a[i]
    var first_vis: [0..=n]?Int = none
    var cur_best = ai - 1
    for j in i..n:
        let aj = a[j]
        if aj < ai: continue
        if aj == ai:
            cur_best += 1
            max_len[tidx(i, j)] = cur_best
        elif aj <= cur_best + 1:
            if first_vis[aj] == none: first_vis[aj] = j
            cur_best max= max_len[tidx(first_vis[aj]!, j)]
    if i == 0: print(cur_best)
