// Ideal CP Lang program for cf2245d1 (D1. Construct an Array (Easy Version)).
// Not gated: upstream ecnerwala/contest-programming CF/2245/D1/D1.cpp is an
// 8-line empty template with no solution.

@!multitest
let (n, m): (1..=1000, 1..=500_500) = read()
domain V = 1..=n

var nonneg = Grid<V, V, Bool>(false)
for _ in 0..m:
    let (o, i, j): (1..=2, V, V) = read()
    let ok = (o == 1)
    nonneg[i, j] = ok
    nonneg[j, i] = ok

var pos = List<V>()
var neg = List<V>()
for v in V:
    if nonneg[v, v]: pos.push(v) else: neg.push(v)

// For any valid array:
//   - two non-negative elements always sum to >= 0
//   - two negative elements always sum to < 0
//   - a_u + a_v >= 0 (u in pos, v in neg) iff a_u >= |a_v|,
//     so the bipartite matrix must be a Ferrers / chain graph.
// Assign a_u = deg(u) for u in pos, and a_v = -(pos.len - deg(v) + 1) for v in neg,
// then verify all m pairs.
var ans: [V]Int = 0
for u in pos:
    var deg = 0
    for v in neg:
        if nonneg[u, v]: deg += 1
    ans[u] = deg

for v in neg:
    var beaten_by = 0
    for u in pos:
        if nonneg[u, v]: beaten_by += 1
    ans[v] = -(pos.len - beaten_by + 1)

var valid = true
for i in V:
    for j in i..:
        if (ans[i] + ans[j] >= 0) != nonneg[i, j]:
            valid = false

if valid:
    print("YES")
    print(ans)
else:
    print("NO")
