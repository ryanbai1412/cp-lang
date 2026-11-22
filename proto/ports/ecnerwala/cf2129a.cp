// Codeforces 2129 A — Double Perspective
//
// Each pair `(a, b)` is an interval `[a, b]` on `1..=2 * n`. We want a subset
// of intervals maximizing union length minus cycle contribution. Grouping by
// left endpoint `a` and keeping only the single interval with the largest right
// endpoint `b` covers every point covered by any input interval (maximal union)
// while ensuring all chosen left endpoints are distinct — hence the endpoint
// graph is a forest with zero cycles.

@!multitest
let n: 1..=3_000 = read()
domain Pos = 1..=2 * n
domain Id = 1..=n
var best_end: [Pos]?(end: Pos, id: Id) = none
for id in Id:
    let (a, b): (Pos, Pos) = read()
    best_end[a] max= (end: b, id:)

let ans = [p.id for x in best_end if let p = x]
print(ans.len)
print(ans)
