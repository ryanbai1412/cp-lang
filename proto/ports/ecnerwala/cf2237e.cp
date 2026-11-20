// Codeforces 2237 E: find a permutation b commuting with a (b[a[i]] == a[b[i]])
// by matching untouched cycles of equal length, then propagating forced values.
@!multitest
let n: Int = read()
domain I = 0..n
let a_raw: [I]Int = read()
let a = [I] i => I(a_raw[i] - 1)
let b_raw: [I]Int = read()
var b: [I]?I = [I] i => if b_raw[i] == -1: none else: I(b_raw[i] - 1)

fn solve() -> Bool:
    var used_val: [I]Bool = false
    for i in I:
        if let v = b[i]: used_val[v] = true
    var vis: [I]Bool = false
    var unused_ind: List<List<I>> = List.filled(n + 1, [])
    var unused_val: List<List<I>> = List.filled(n + 1, [])
    for i in I:
        if vis[i]: continue
        var len = 0
        var j = i
        var has_ind = false
        var has_val = false
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
        for z in 0..unused_ind[len].len:
            b[unused_ind[len][z]] = unused_val[len][z]
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
    print([b[i]!.ord + 1 for i in I])
else:
    print("NO")
