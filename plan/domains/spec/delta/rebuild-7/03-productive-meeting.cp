// Codeforces 1579D Productive Meeting
// in: t; per test n, then sociabilities a_1..a_n. out: per test k = max talks, then k lines "i j" (1-based people).
@!multitest
domain P = 1..=read<2..=2e5>()
let a = read<[P]0..=2e5>()
var h: Heap<(Int, P)> = Heap.of([(a[p], p) for p in P if a[p] > 0], by: desc)
var talks: List<(P, P)> = []
while h.len >= 2:
    let (x, i) = h.pop()!
    let (y, j) = h.pop()!
    talks.push((i, j))
    if x > 1: h.push((x - 1, i))
    if y > 1: h.push((y - 1, j))
print(talks.len)
for (i, j) in talks: print(i, j)
