// Source: DMOPC '21 Contest 2 P2 (Scrambling Swaps), https://dmoj.ca/problem/dmopc21c2p2
// limits: 2000 ms, 256 MB
// Problem: a list of swaps (x, y) of positions of an array of length n <= 300 starts empty; q <=
//   3e5 queries: "B x y" adds a swap at the front, "E x y" at the end, "Q p" asks for any starting
//   permutation that the list, applied in order, turns into p (at most 3000 such queries).
// Solution: keep the whole list as one permutation perm (final[i] = start[perm[i]]) and its
//   inverse. A swap appended at the end swaps two positions of perm; one added at the front swaps
//   two values (positions inv[x], inv[y]). Answer Q with start[v] = p[inv[v]]. O(1) per swap, O(n)
//   per Q.
// original: corpus/submissions/3946515.cpp
// Bounds: n: 1..=300; q: 1..=3e5; u, v, i, j, target: 1..=n
let n: 1..=300 = read()
let q: 1..=3e5 = read()
domain Pos = 1..=n
domain Val = 1..=n

// final[i] = start[perm[i]]; at_pos is perm's inverse
var perm: [Pos]Val = [Pos] i => Val.nth(i.ord)
var at_pos: [Val]Pos = [Val] v => Pos.nth(v.ord)

fn swap_positions(i: Pos, j: Pos):
    let (u, v) = (perm[i], perm[j])
    perm[i] = v
    perm[j] = u
    at_pos[u] = j
    at_pos[v] = i

for _ in 0..q:
    let kind: Str = read()
    if kind == "B":
        let u: Val = read()
        let v: Val = read()
        swap_positions(at_pos[u], at_pos[v])
    elif kind == "E":
        let i: Pos = read()
        let j: Pos = read()
        swap_positions(i, j)
    else:
        let target: [Pos]1..=n = read()
        print([target[at_pos[v]] for v in Val])
