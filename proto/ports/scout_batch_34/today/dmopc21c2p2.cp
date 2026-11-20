// Runnable port (proto/cp.py lib-run) of dmopc21c2p2.cp
// original: corpus/submissions/3946515.cpp

let (n, q): (Int, Int) = read()
domain Pos = 1..=n
domain Val = 1..=n

var perm: [Pos]Val = [Pos] p => Val(p.to_idx + 1)
var inv: [Val]Pos = [Val] v => Pos(v.to_idx + 1)

fn swap_pos(i: Pos, j: Pos):
    if i == j: return
    let u = perm[i]
    let v = perm[j]
    perm[i] = v
    perm[j] = u
    inv[u] = j
    inv[v] = i

fn swap_val(u: Val, v: Val):
    swap_pos(inv[u], inv[v])

for _ in 0..q:
    let kind: Str = read()
    if kind == "B":
        let (u, v): (Val, Val) = read()
        swap_val(u, v)
    elif kind == "E":
        let (i, j): (Pos, Pos) = read()
        swap_pos(i, j)
    else:
        let target: [Pos]Int = read()
        var out = ""
        for v in Val:
            if v.to_idx > 0: out = out + " "
            out = out + "{target[inv[v]]}"
        print(out)
