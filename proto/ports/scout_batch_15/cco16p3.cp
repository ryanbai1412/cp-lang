// CCO '16 P3 — Legends (corpus/submissions/5568296.cpp)
// Five subtasks of forbidden subgraph / graph-connectivity detection on
// undirected simple graphs, combining cycle-chords via DSU, DFS back-edge
// depth checks, and vertex-biconnected components (`biconnected_components`).

use std.dsu.{Dsu}
use std.graph.{Graph}
use scout.connectivity.{biconnected_components}

fn solve_s1():
    let (n, m): (Int, Int) = read()
    domain V = 1..=n
    domain E = 0..m
    let edges: [E](u: V, v: V) = read()
    let g = Graph<V, E>.undirected(edges)

    // Iterative DFS tree with explicit ancestor path stack.
    var tin: [V]?Int = none
    var out = [V] u => [a.to for a in g.adj(u)]
    var stack = List<V>()
    var pos_in_stack: [V]?Int = none
    var frames = List<(V, Int, ?V)>()
    var timer = 0
    var ans = false

    for root in V:
        if tin[root] != none or ans: continue
        frames.push((root, 0, none))
        while not frames.is_empty and not ans:
            let (u, i, p) = frames.last
            if i == 0:
                tin[u] = timer
                timer += 1
                pos_in_stack[u] = stack.len
                stack.push(u)
            if i < out[u].len:
                frames[frames.len - 1] = (u, i + 1, p)
                let v = out[u][i]
                if p != none and v == p!: continue
                if tin[v] == none:
                    frames.push((v, 0, u))
                elif tin[u]! > tin[v]!:
                    let start = pos_in_stack[v]!
                    var on_cyc: [V]Bool = false
                    for k in start..stack.len: on_cyc[stack[k]] = true
                    var dsu = Dsu<V>.new()
                    for e in E:
                        let (eu, ev) = edges[e]
                        if not (on_cyc[eu] and on_cyc[ev]):
                            dsu.unite(eu, ev)
                    var seen_root: [V]Bool = false
                    for k in start..stack.len:
                        let r = dsu.find(stack[k])
                        if seen_root[r]: ans = true
                        seen_root[r] = true
            else:
                frames.pop()
                stack.pop()
                pos_in_stack[u] = none
    print(if ans: "YES" else: "NO")

fn solve_s2():
    let (n, m): (Int, Int) = read()
    domain V = 1..=n
    domain E = 0..m
    let edges: [E](u: V, v: V) = read()
    var dsu = Dsu<V>.new()
    var has_cycle = false
    for e in E:
        if not dsu.unite(edges[e].u, edges[e].v): has_cycle = true
    print(if has_cycle: "YES" else: "NO")

fn solve_s3():
    let (n, m): (Int, Int) = read()
    domain V = 1..=n
    domain E = 0..m
    let edges: [E](u: V, v: V) = read()
    let g = Graph<V, E>.undirected(edges)
    let out = [V] u => [a.to for a in g.adj(u)]
    var tin: [V]?Int = none
    var depth: [V]Int = 0
    var par: [V]?V = none
    var up: [V]Bool = false
    var frames = List<(V, Int, ?V)>()
    var timer = 0
    var ans = false

    for root in V:
        if tin[root] != none: continue
        frames.push((root, 0, none))
        while not frames.is_empty:
            let (u, i, p) = frames.last
            if i == 0:
                tin[u] = timer
                timer += 1
                par[u] = p
            if i < out[u].len:
                frames[frames.len - 1] = (u, i + 1, p)
                let v = out[u][i]
                if p != none and v == p!: continue
                if tin[v] == none:
                    depth[v] = depth[u] + 1
                    frames.push((v, 0, u))
                elif tin[u]! > tin[v]!:
                    up[u] = true
                    if depth[u] - depth[v] >= 3: ans = true
            else:
                frames.pop()

    var up_kids: [V]Int = 0
    for u in V:
        if let p = par[u]:
            if up[u]:
                up_kids[p] += 1
                if up[p]: ans = true
    for u in V:
        if up_kids[u] >= 2: ans = true
    print(if ans: "YES" else: "NO")

fn solve_s4():
    let (n, m): (Int, Int) = read()
    domain V = 1..=n
    domain E = 0..m
    let edges: [E](u: V, v: V) = read()
    var deg: [V]Int = 0
    for e in E:
        deg[edges[e].u] += 1
        deg[edges[e].v] += 1
    var ans = false
    for u in V:
        if deg[u] >= 3: ans = true
    print(if ans: "YES" else: "NO")

fn solve_s5():
    let (n, m): (Int, Int) = read()
    domain V = 1..=n
    domain E = 0..m
    let edges: [E](u: V, v: V) = read()
    let g = Graph<V, E>.undirected(edges)
    var has_edge = [V] _ => [V] _ => false
    var deg: [V]Int = 0
    for e in E:
        let (u, v) = edges[e]
        has_edge[u][v] = true
        has_edge[v][u] = true
        deg[u] += 1
        deg[v] += 1

    // Check whether `cyc` is a cycle whose vertices have >= 2 external exits
    // (for 3- and 4-cycles, avoiding the single-shared-apex case).
    fn check_cycle(cyc: List<V>, guard_apex: Bool) -> Bool:
        let k = cyc.len
        for i in 0..k:
            if not has_edge[cyc[i]][cyc[(i + 1) % k]]: return false
        var out_nodes = 0
        var deg_max = 0
        for u in cyc:
            var ext = deg[u]
            for v in cyc:
                if has_edge[u][v]: ext -= 1
            deg_max max= ext
            if ext > 0: out_nodes += 1
        if guard_apex and out_nodes >= 2 and deg_max == 1:
            var in_cyc: [V]Bool = false
            for u in cyc: in_cyc[u] = true
            var ext_seen: [V]Bool = false
            var ext_uniq = 0
            for u in cyc:
                for a in g.adj(u):
                    if not in_cyc[a.to] and not ext_seen[a.to]:
                        ext_seen[a.to] = true
                        ext_uniq += 1
            if ext_uniq <= 1: return false
        out_nodes >= 2

    var ans = false
    for i in 0..m:
        if ans: break
        let (u1, v1) = edges[E(i)]
        for j in i + 1..m:
            let (u2, v2) = edges[E(j)]
            var pts = [u1, v1, u2, v2]
            var uniq = List<V>()
            for x in pts:
                var dup = false
                for y in uniq:
                    if x == y: dup = true
                if not dup: uniq.push(x)
            if uniq.len == 3:
                if check_cycle(uniq, true): ans = true
            elif uniq.len == 4:
                if check_cycle([u1, v1, u2, v2], true) or check_cycle([u1, v1, v2, u2], true):
                    ans = true
            if ans: break

    if not ans:
        for block in biconnected_components::<V>(u => g.adj(u)):
            if block.len == 5:
                // Fix block[0] and test all 24 permutations of block[1..5].
                let a0 = block[0]
                for i1 in 1..5:
                    for i2 in 1..5:
                        if i2 == i1: continue
                        for i3 in 1..5:
                            if i3 == i1 or i3 == i2: continue
                            let i4 = 10 - i1 - i2 - i3
                            if check_cycle([a0, block[i1], block[i2], block[i3], block[i4]], false):
                                ans = true
            elif block.len > 5:
                var high = 0
                for u in block:
                    if deg[u] > 2: high += 1
                if high >= 2: ans = true
    print(if ans: "YES" else: "NO")

let (c, tcs): (Int, Int) = read()
for _ in 0..tcs:
    if c == 1: solve_s1()
    elif c == 2: solve_s2()
    elif c == 3: solve_s3()
    elif c == 4: solve_s4()
    else: solve_s5()
