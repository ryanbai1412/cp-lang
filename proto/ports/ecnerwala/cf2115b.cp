@!multitest
let (n, q): (Int, Int) = read()
let b: List<Int> = [read::<Int>() for _ in 0..n]
let nv = n + q
var cur_vars: [0..n]Int = 0
for i in 0..n: cur_vars[i] = i
var in_e: [0..nv]List<Int> = []
var out_e: [0..nv]List<Int> = []
var indeg: [0..nv]Int = 0
for i in 0..n: indeg[i] = 1
for v in n..nv:
    let (x0, y0, z0): (Int, Int, Int) = read()
    let (x, y, z) = (x0 - 1, y0 - 1, z0 - 1)
    let cx = cur_vars[x]
    let cy = cur_vars[y]
    in_e[v].push(cx)
    in_e[v].push(cy)
    out_e[cx].push(v)
    out_e[cy].push(v)
    cur_vars[z] = v
    indeg[v] = 2
var alive: [0..nv]Bool = true
var ans: [0..nv]Int = 0
var evts: List<(Int, Int)> = [(b[i], cur_vars[i]) for i in 0..n]
evts.sort()
evts.reverse()
var stk: List<Int> = []
for (val, st) in evts:
    if not alive[st]:
        if ans[st] == val: continue
        print(-1)
        return
    alive[st] = false
    ans[st] = val
    stk.push(st)
    while let cur = stk.pop():
        for nxt in in_e[cur]:
            if alive[nxt]:
                alive[nxt] = false
                ans[nxt] = val
                stk.push(nxt)
        for nxt in out_e[cur]:
            indeg[nxt] -= 1
            if indeg[nxt] == 0 and alive[nxt]:
                alive[nxt] = false
                ans[nxt] = val
                stk.push(nxt)
print([ans[i] for i in 0..n])
