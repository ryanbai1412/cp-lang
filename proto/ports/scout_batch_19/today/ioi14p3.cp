// IOI 2014 P3, Game (reversed callback interactive, stdio-adapted today-form)
// original: corpus/submissions/4683972.cpp
let n: 2..=1500 = read()
domain V = 0..n

var par: [V]Int = -1
var comp_sz: [V]Int = 1
var edge_cnt: [V, V]Int = 0

fn root_of(mut p: [V]Int, x: V) -> V:
    var cur = x
    while p[cur] >= 0:
        cur = V(p[cur])
    var y = x
    while p[y] >= 0:
        let nxt = V(p[y])
        p[y] = cur.label
        y = nxt
    cur

fn has_edge(mut p: [V]Int, mut sz: [V]Int, mut cnt: [V, V]Int, u0: V, v0: V) -> Int:
    let u = root_of(mut p, u0)
    let v = root_of(mut p, v0)
    cnt[u, v] += 1
    cnt[v, u] += 1
    if cnt[u, v] == sz[u] * sz[v]:
        sz[v] += sz[u]
        p[u] = v.label
        for i in V:
            cnt[v, i] += cnt[u, i]
        for i in V:
            if i != v:
                cnt[i, v] += cnt[i, u]
        return 1
    0

let total_pairs = n * (n - 1) / 2
var reply = 0
for step_i in 0..total_pairs:
    if step_i == 0:
        let e: (V, V) = ask("READY")
        reply = has_edge(mut par, mut comp_sz, mut edge_cnt, e.0, e.1)
    else:
        let e: (V, V) = ask(reply)
        reply = has_edge(mut par, mut comp_sz, mut edge_cnt, e.0, e.1)
print(reply)
