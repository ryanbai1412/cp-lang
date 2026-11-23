// Codeforces 2237 E: find a permutation b commuting with a (b[a[i]] == a[b[i]]).
// Commutation means b maps each cycle of a onto a cycle of a of the same length
// preserving step order: fixing b[i] forces b[a^k(i)] = a^k(b[i]). Untouched
// index cycles must pair bijectively by length with untouched value cycles;
// once one representative per untouched cycle is paired, every remaining cycle
// has a fixed entry, and propagating b[a[i]] = a[b[i]] either closes consistently
// (by total-size balance, no cycle can wrap onto a shorter one without another
// failing to close on a longer one) or detects a conflict.
@!multitest
let n: 1..=500_000 = read()
domain I = 1..=n
domain Len = 1..=n
let a: [I]I = read()
var b: [I]?I = read(none: -1)

fn cycles(of: [I]I) -> List<List<I>>:
    var vis: [I]Bool = false
    var res: List<List<I>> = []
    for st in I if not vis[st]:
        var cyc: List<I> = []
        var j = st
        while not vis[j]:
            vis[j] = true
            cyc.push(j)
            j = of[j]
        res.push(cyc)
    res

fn solve() -> ?[I]I:
    var used_val: [I]Bool = false
    for i in I:
        if let v = b[i]: used_val[v] = true

    var unused_ind: [Len]List<I> = []
    var unused_val: [Len]List<I> = []
    for cyc in cycles(of: a):
        let len = Len(cyc.len)
        if not cyc.any(i => b[i] != none): unused_ind[len].push(cyc[0])
        if not cyc.any(i => used_val[i]): unused_val[len].push(cyc[0])

    for len in Len:
        if unused_ind[len].len != unused_val[len].len: return none
        for k in 0..unused_ind[len].len: b[unused_ind[len][k]] = some(unused_val[len][k])

    for st in I if b[st] != none:
        var (i, v) = (st, b[st]!)
        while b[a[i]] == none:
            (i, v) = (a[i], a[v])
            b[i] = v
        if b[a[i]] != a[v]: return none

    [I] i => b[i]!

if let ans = solve():
    print("YES")
    print(ans)
else:
    print("NO")
