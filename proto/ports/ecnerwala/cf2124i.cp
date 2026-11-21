@!multitest
let n: Int = read()
let x: List<Int> = [read::<Int>() for _ in 0..n]
var stk: [0..(n + 1)]Int = -1
stk[x[0]] = 0
var par: [0..n]Int = -1
var is_ext: [0..n]Bool = false
for i in 1..n:
    if x[i] > x[i - 1] + 1 or x[i] == 1:
        print("NO")
        return
    par[i] = stk[x[i] - 1]
    if x[i] <= x[i - 1]: is_ext[par[i]] = true
    stk[x[i]] = i
for i in 1..n:
    if is_ext[i] and is_ext[par[i]]:
        print("NO")
        return
var nxt: [0..n]Int = -1
var hd: [0..n]Int = -1
var tl: [0..n]Int = -1
fn merge(ah: Int, at: Int, bh: Int, bt: Int) -> (Int, Int):
    if at == -1: return (bh, bt)
    if bt == -1: return (ah, at)
    nxt[at] = bh
    (ah, bt)
for i in (0..n).rev:
    if is_ext[i]:
        let (h, t) = merge(i, i, hd[i], tl[i])
        hd[i] = h
        tl[i] = t
    else:
        let (h, t) = merge(hd[i], tl[i], i, i)
        hd[i] = h
        tl[i] = t
    if i == 0: break
    let p = par[i]
    let (h, t) = merge(hd[i], tl[i], hd[p], tl[p])
    hd[p] = h
    tl[p] = t
var res = List.filled(n, 0)
var cur = hd[0]
for v in 0..n:
    res[cur] = v + 1
    cur = nxt[cur]
print("YES")
print(res)
