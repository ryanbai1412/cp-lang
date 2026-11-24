// Source: Codeforces 2245D2 (Construct an Array, Hard Version),
//   https://codeforces.com/contest/2245/problem/D2
// limits: 4000 ms, 1024 MB
// Problem: find integers a[1..n], |a| <= 1e9, meeting m constraints (o, i, j), i <= j: o = 1
//   needs a[i] + a[j] >= 0, o = 2 needs a[i] + a[j] < 0; print YES and a, or NO.
//   n <= 2e5, m <= min(1e6, n(n + 1) / 2), each pair i <= j at most once; multitest, sum of
//   n <= 2e5, sum of m <= 1e6.
// Solution: peel any vertex whose remaining constraints are all one kind and give the z-th
//   peeled vertex magnitude n - z, positive for kind 1, negative for kind 2: it outweighs every
//   later neighbour, so its sign settles all its remaining constraints (self-loops included).
//   A queue of ready vertices with per-kind counts; if peeling stalls, print NO. O(n + m).
// Bounds: n: 1..=2e5; m: 0..=min(1e6, n * (n + 1) / 2); raw.kind: 1..=2; raw.u, raw.v: 1..=n
@!multitest
let n: 1..=2e5 = read()
let m: 0..=min(1e6, n * (n + 1) / 2) = read()
domain V = 1..=n
domain E = 0..m
domain Kind = 1..=2
let raw: [E](kind: Kind, u: V, v: V) = read()
let g = Graph.undirected([E] e => (u: raw[e].u, v: raw[e].v))

fn solve() -> ?[V]Int:
    var cnt: [V, Kind]Int = 0
    for e in E:
        cnt[raw[e].u, raw[e].kind] += 1
        cnt[raw[e].v, raw[e].kind] += 1

    fn ready(v: V) -> Bool = cnt[v, Kind(1)] == 0 or cnt[v, Kind(2)] == 0

    var order = [v for v in V if ready(v)]
    var queued: [V]Bool = [V] v => ready(v)
    var ans: [V]Int = 0
    var z = 0
    while z < order.len:
        let v = order[z]
        let mag = n - z
        ans[v] = if cnt[v, Kind(1)] == 0: -mag else: mag
        for a in g.out(v) if not queued[a.to]:
            cnt[a.to, raw[a.e].kind] -= 1
            if ready(a.to):
                order.push(a.to)
                queued[a.to] = true
        z += 1

    if order.len < n: None else: ans

if let ans = solve():
    print("YES")
    print(ans)
else:
    print("NO")
