// Source: Codeforces 2237E (Permutation Commutation),
//   https://codeforces.com/contest/2237/problem/E
// Problem: a permutation a of 1..n and b with some entries -1 (the given values distinct).
//   Complete b to the lexicographically smallest permutation with a[b[i]] = b[a[i]] for all
//   i: print YES and it, or NO. n <= 2e5 (sum), multitest.
// Solution: commuting means b maps each cycle of a onto a same-length cycle step by step:
//   b[a^k(i)] = a^k(b[i]). Cycles with no fixed entry must pair by length with cycles none
//   of whose values are used; pair them smallest-first (the k-th smallest index cycle gets
//   the k-th smallest value cycle, minimum to minimum), else NO. Then from every fixed entry
//   walk b[a[i]] = a[b[i]] until a filled entry, which must agree. O(n).
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
        if not cyc.any(i => b[i] != None): unused_ind[len].push(cyc[0])
        if not cyc.any(i => used_val[i]): unused_val[len].push(cyc[0])

    for len in Len:
        if unused_ind[len].len != unused_val[len].len: return None
        for k in 0..unused_ind[len].len: b[unused_ind[len][k]] = Some(unused_val[len][k])

    for st in I if b[st] != None:
        var (i, v) = (st, b[st]!)
        while b[a[i]] == None:
            (i, v) = (a[i], a[v])
            b[i] = v
        if b[a[i]] != a[v]: return None

    [I] i => b[i]!

if let ans = solve():
    print("YES")
    print(ans)
else:
    print("NO")
