// expect: ok, output "YES"
// `PalindromicTree.pop` after random pushes and pops leaves the tree a fresh build of the same string gives.
use std.strings.{PalindromicTree}

domain L = 0..3
var seed = 2468
fn next_rand(m: Int) -> Int:
    seed = (seed * 1103515245 + 12345) & 2_147_483_647
    (seed >> 8) % m

var ok = true
for _ in 0..200:
    var t = PalindromicTree<L>.new()
    var s: List<L> = []
    for _ in 0..next_rand(40):
        if s.len > 0 and next_rand(3) == 0:
            t.pop()
            let _ = s.pop()
        else:
            let a = L(next_rand(3))
            let _ = t.push(a)
            s.push(a)
        let f = PalindromicTree<L>.of(s)
        ok = ok and t.distinct == f.distinct and t.last_state == f.last_state and t.occ_counts() == f.occ_counts()
        for v in 0..f.len: ok = ok and t.pal_len(v) == f.pal_len(v) and t.link(v) == f.link(v)
        for v in 0..f.len:
            for a in L: ok = ok and t.step(v, a) == f.step(v, a)
    if s.len > 0:
        let _ = t.push_front(L(0))
        let _ = t.push(L(1))
        t.pop()
        s.insert(0, L(0))
        let f = PalindromicTree<L>.of(s)
        ok = ok and t.distinct == f.distinct
print(if ok: "YES" else: "NO")
