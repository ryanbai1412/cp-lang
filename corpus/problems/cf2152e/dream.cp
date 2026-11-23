// Codeforces 2152 E — Monotone Subsequence (interactive)
//
// By Erdős–Szekeres, every permutation of length `len = n * n + 1` contains a
// monotone subsequence of length `n + 1`. Each query on the remaining
// unassigned positions (`layer[v] == none`) returns their left-to-right maxima
// `got`, which form a strictly increasing subsequence:
// - If `got.len > n`, its first `n + 1` positions already form an increasing
//   subsequence of length `n + 1`, so print them and finish immediately.
// - Otherwise record `layer[v] = i` for every `v in got`. Every position left
//   unassigned after round `i` must be smaller than some position in layer `i`
//   to its left.
// After `n` rounds, at least one position remains unassigned (`layer[v] == none`).
// Walking backwards from that position through layers `n - 1` down to `0`
// always finds a larger element to the left, yielding a decreasing subsequence
// of length `n + 1`.

@!interactive
@!multitest
let n: 1..=100 = read()
domain Pos = 1..=n * n + 1
var layer: [Pos]?Int = None

for i in 0..n:
    let q = [v for v in Pos if layer[v] == None]
    print("?", q.len, q)
    let cnt: Int = read()
    let got: [0..cnt]Pos = read()
    for v in got: layer[v] = i
    if cnt > n:
        print("!", [v for v in Pos if layer[v] == i][..=n])
        return

var cur = Pos.first(u => layer[u] == None)!
var ans: List<Pos> = [cur]
for j in (0..n).rev:
    while layer[cur] != j: cur = cur.prev!
    ans.push(cur)
ans.reverse()
print("!", ans)
