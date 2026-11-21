// Codeforces 2237 E: find a permutation b commuting with a (b[a[i]] == a[b[i]])
// by matching untouched cycles of equal length, then propagating forced values.
@!multitest
let n: Int = read()
domain I = 1..=n
let a: [I]I = read()
let b_raw: [I]Int = read()
var b: [I]?I = [I] i => if b_raw[i] == -1: none else: I(b_raw[i])

fn solve() -> Bool:
    var used_val: [I]Bool = false
    for i in I:
        if let v = b[i]: used_val[v] = true
    var vis: [I]Bool = false
    var unused_ind: [1..=n]List<I> = []
    var unused_val: [1..=n]List<I> = []
    for i in I:
        if vis[i]: continue
        var (len, j, has_ind, has_val) = (0, i, false, false)
        while true:
            vis[j] = true
            len += 1
            if b[j] != none: has_ind = true
            if used_val[j]: has_val = true
            j = a[j]
            if j == i: break
        if not has_ind: unused_ind[len].push(i)
        if not has_val: unused_val[len].push(i)
    for len in 1..=n:
        if unused_ind[len].len != unused_val[len].len: return false
        for (u, v) in zip(unused_ind[len], unused_val[len]): b[u] = v
    for st in I:
        if b[st] == none: continue
        var i = st
        while true:
            let cur = b[i]!
            let nxt = a[i]
            if let existing = b[nxt]:
                if existing != a[cur]: return false
                break
            b[nxt] = a[cur]
            i = nxt
    true

if solve():
    print("YES")
    print([b[i]! for i in I])
else:
    print("NO")
