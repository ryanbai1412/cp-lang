// CCO '22 P4 — Good Groups / Binary Tree Parking (DMOJ: cco22p4)
// original: corpus/submissions/4801884.cpp
// CP Lang v2 (locked spec, docs/one-page-cheat-sheet.md @ 3723dca)
//
// Given a rooted binary tree of n nodes (1..=n, missing child encoded as 0)
// and m items initially assigned to nodes, count valid orderings modulo 1e9+7.
//
// `Mod<MOD>`, `Comb<MOD>`, `read(none: 0)`, and `@memo` replace the original's
// manual modular inverse, factorial arrays, `-1` child bit-hack, and raw
// `memset(memo, -1, ...)` table.
use std.math.{Comb}

const MOD = 1_000_000_007
type Z = Mod<MOD>

let (n, m): (1..=4000, 0..=4000) = read()
domain V = 1..=n
let comb = Comb<MOD>(n)

var here: [V]Int = 0
for _ in 0..m:
    let u: V = read()
    here[u] += 1

let kids: [V](l: ?V, r: ?V) = read(none: 0)

var is_right: [V]Bool = false
var sub_size: [V]Int = 0
var sub_items: [V]Int = 0
var spare: [V]Int = 0

fn prep(u: V):
    sub_size[u] = 1
    sub_items[u] = here[u]
    if let l = kids[u].l:
        is_right[l] = false
        prep(l)
        sub_size[u] += sub_size[l]
        sub_items[u] += sub_items[l]
    if let r = kids[u].r:
        is_right[r] = true
        prep(r)
        sub_size[u] += sub_size[r]
        sub_items[u] += sub_items[r]
    spare[u] = sub_size[u] - sub_items[u]

prep(V.first)
if min(spare[u] for u in V)! < 0:
    print(0)
    return

@memo
fn dp(u: V, k: 0..=n) -> Z:
    if k > spare[u]: return 0
    let (lo, ro) = kids[u]
    if let l = lo:
        if let r = ro:
            let (first, second) = if is_right[u]: (r, l) else: (l, r)
            var ans: Z = 0
            for i in 0..=here[u]:
                let bl = i
                let br = here[u] - i
                let al = min(k, spare[first] - bl)
                let ar = k - al
                if al >= 0:
                    let xr = if al + bl == spare[first] and ar + br > spare[second]: ar + br - 1
                             else: ar + br
                    ans += comb.choose(here[u], i)
                        * comb.arrange(al + bl, bl)
                        * comb.arrange(ar + br, br)
                        * dp(first, al + bl)
                        * dp(second, xr)
            return ans
        else:
            let next_k = if k == spare[u]: k + here[u] - 1 else: k + here[u]
            return comb.arrange(k + here[u], here[u]) * dp(l, next_k)
    elif let r = ro:
        let next_k = if k == spare[u]: k + here[u] - 1 else: k + here[u]
        return comb.arrange(k + here[u], here[u]) * dp(r, next_k)
    else:
        return 1

print(dp(V.first, 0))
