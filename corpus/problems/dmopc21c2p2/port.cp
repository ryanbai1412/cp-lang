// Source: DMOPC '21 Contest 2 P2 (Scrambling Swaps), https://dmoj.ca/problem/dmopc21c2p2
// Problem: a list of swaps (x, y) of positions of an array of length n <= 300 starts empty; q <=
//   3e5 queries: "B x y" adds a swap at the front, "E x y" at the end, "Q p" asks for any starting
//   permutation that the list, applied in order, turns into p (at most 3000 such queries).
// Solution: keep the whole list as one permutation perm (final[i] = start[perm[i]]) and its
//   inverse. A swap appended at the end swaps two positions of perm; one added at the front swaps
//   two values (positions inv[x], inv[y]). Answer Q with start[v] = p[inv[v]]. O(1) per swap, O(n)
//   per Q.
// original: corpus/submissions/3946515.cpp

let (n, q): (Int, Int) = read()
domain Pos = 1..=n
domain Val = 1..=n

var perm: [Pos]Val = [Pos] p => Val(p.ord + 1)
var inv: [Val]Pos = [Val] v => Pos(v.ord + 1)

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
            if v.ord > 0: out += " "
            out += "{target[inv[v]]}"
        print(out)
