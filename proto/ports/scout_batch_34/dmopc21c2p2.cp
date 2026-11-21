// Port of dmopc21c2p2 (DMOPC '21 Contest 2 P2 - Scavenger Hunt)
// original: corpus/submissions/3946515.cpp
// Algorithm: Maintain the net permutation of swaps prepended ('B') and
// appended ('E') via a Pos <-> Val bijection, then apply its inverse on 'Q'.

let (n, q): (Int, Int) = read()
domain Pos = 1..=n
domain Val = 1..=n

var perm: [Pos]Val = [Pos] p => Val(p.to_idx + 1)
var inv: [Val]Pos = [Val] v => Pos(v.to_idx + 1)

// FRICTION UD8: Swapping two elements of the same domain-indexed array
// (`swap(perm[i], perm[j])`) is a same-container mutable alias; when `i == j`
// it is a no-op, yet strict disjoint-borrow checking requires a guard or a
// built-in `.swap(i, j)` method on indexed arrays.
fn swap_pos(i: Pos, j: Pos):
    if i == j: return
    let (u, v) = (perm[i], perm[j])
    perm[i] = v
    perm[j] = u
    inv[u] = j
    inv[v] = i

fn swap_val(u: Val, v: Val) = swap_pos(inv[u], inv[v])

for _ in 0..q:
    let kind: Char = read()
    match kind:
        'B' =>
            let (u, v): (Val, Val) = read()
            swap_val(u, v)
        'E' =>
            let (i, j): (Pos, Pos) = read()
            swap_pos(i, j)
        _ =>
            let target: [Pos]Int = read()
            var ans: [Val]Int = [Val] _ => 0
            for v in Val:
                ans[v] = target[inv[v]]
            print([ans[v] for v in Val])
