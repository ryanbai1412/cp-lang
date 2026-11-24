// Source: AtCoder ABC478 E (Inequality Sequence Construction),
//   https://atcoder.jp/contests/abc478/tasks/abc478_e
// Problem: find A of length n with values in 1..=n meeting q conditions A[u] <= A[v] (t = 0) or
//   A[u] < A[v] (t = 1); print Yes and A, or No. n, q <= 2e5.
// Solution: conditions are arcs u -> v of weight t. A strict arc inside a strongly connected
//   component is a contradiction; otherwise A is the longest path (from 1) in the condensation.
//   `scc` gives components in topological order, so one DP pass relaxes dp[to] max= dp[c] + t,
//   O(n + q); values stay <= the component count <= n.

let (n, q): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 0..q
let conds: [E](t: 0..=1, u: V, v: V) = read()

let arcs = [E] e => (u: conds[e].u, v: conds[e].v)
let g = Graph.directed(arcs)
let (C, comp) = scc(g)

for e in E:
    if conds[e].t == 1 and comp[conds[e].u] == comp[conds[e].v]:
        print("No")
        return

var c_out: [C]List<(C, Int)> = []
for e in E:
    let cu = comp[conds[e].u]
    let cv = comp[conds[e].v]
    if cu != cv:
        c_out[cu].push((cv, conds[e].t))

var dp: [C]Int = 1
for c in C:
    for (to, w) in c_out[c]:
        dp[to] max= dp[c] + w

print("Yes")
print([dp[comp[v]] for v in V])

@alt(program)
fn slow():
    var d: [V]Int = 1
    for step in 0..=n:
        var changed = false
        for e in E:
            let u = conds[e].u
            let v = conds[e].v
            let w = conds[e].t
            if d[u] + w > d[v]:
                d[v] = d[u] + w
                changed = true
                if step == n:
                    print("No")
                    return
        if not changed:
            break
    print("Yes")
    print(d)

@check
fn check():
    // Exact Bellman-Ford oracle for feasibility (longest path with weights 0/1)
    var d: [V]Int = 1
    var possible = true
    for step in 0..=n:
        var changed = false
        for e in E:
            let u = conds[e].u
            let v = conds[e].v
            let w = conds[e].t
            if d[u] + w > d[v]:
                d[v] = d[u] + w
                changed = true
                if step == n:
                    possible = false
        if not changed:
            break

    let verdict: Str = read_output()
    if not possible:
        assert verdict == "No"
        return
    assert verdict == "Yes"
    let a: [V]Int = read_output()
    for v in V:
        assert 1 <= a[v] and a[v] <= n
    for e in E:
        let u = conds[e].u
        let v = conds[e].v
        if conds[e].t == 0:
            assert a[u] <= a[v]
        else:
            assert a[u] < a[v]
