// CCC '22 S5 - Good Influencers (DMOJ: ccc22s5, 17pt, TL 1.0s, ML 1024MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Tree of n <= 2e5 students. Initially a subset (`Y`/`N`) intends to write CCC.
// Paying student u (cost c[u]) lets u convince all neighbours once u itself
// intends to write CCC. Minimize total cost so every student intends to write.
//
// Bottom-up 3-state tree DP over `Rooted<V>` using `?Int` (none = impossible):
//   Need: u relies on parent(u) to convince u (u may still be paid to push down)
//   Done: u is convinced purely inside its subtree (not required to push up)
//   Push: u is convinced purely inside its subtree AND u is paid (pushes up & down)
use std.graph.{Graph}
use std.tree.{Rooted}

let n: 1..=2e5 = read()
domain V = 1..=n
domain E = 0..(n - 1)
// WISH W18: `Tree<V>` typed input generator is needed so `cpl stress` generates
// connected acyclic graphs rather than arbitrary edge lists.
let edges: [E](u: V, v: V) = read()
let init_str: [V]Char = read() where all(ch == 'Y' or ch == 'N' for ch in init_str)
let cost: [V]1..=1000 = read()

// WISH W19: fieldless enum as a domain/Index for `[V, State]?Int` DP table
enum State:
    Need
    Done
    Push

fn opt_add(a: ?Int, b: ?Int) -> ?Int:
    if let (x, y) = (a, b): x + y else: none

fn opt_min(a: ?Int, b: ?Int) -> ?Int:
    if a == none: b
    elif b == none: a
    else: min(a!, b!)

fn solve() -> Int:
    let g = Graph<V, E>.undirected(edges)
    let tree = Rooted<V>(u => g.out(u), V.first)

    var need: [V]?Int = none
    var done: [V]?Int = none
    var push: [V]?Int = none

    // Bottom-up order: `tree.at` is preorder, so `tree.at` reversed is postorder!
    // WISH W20: `Rooted<V>.postorder()` helper (using `p in Rooted.Pre.rev => tree.at[p]` works but is non-obvious)
    for p in Rooted<V>.Pre.rev:
        let u = tree.at[p]
        let is_y = init_str[u] == 'Y'

        // sum_done: every child satisfies itself (Done or Push)
        // sum_any:  every child is Need, Done, or Push (allowed when u is paid)
        var sum_done: ?Int = 0
        var sum_any: ?Int = 0
        for c in tree.kids[u]:
            sum_done = opt_add(sum_done, opt_min(done[c], push[c]))
            sum_any = opt_add(sum_any, opt_min(need[c], opt_min(done[c], push[c])))

        // State Push: u is paid AND convinced inside its subtree
        if is_y:
            push[u] = opt_add(cost[u], sum_any)
        else:
            // Must pick at least one child c* in Push to ignite u; others can be any
            var best_push: ?Int = none
            if let sa = sum_any:
                for c in tree.kids[u]:
                    if let pc = push[c]:
                        let base = opt_min(need[c], opt_min(done[c], pc))!
                        best_push = opt_min(best_push, sa - base + pc)
            push[u] = opt_add(cost[u], best_push)

        // State Done: u is convinced inside its subtree (paid or unpaid)
        if is_y:
            done[u] = opt_min(sum_done, push[u])
        else:
            var best_unpaid: ?Int = none
            if let sd = sum_done:
                for c in tree.kids[u]:
                    if let pc = push[c]:
                        let base = opt_min(done[c], pc)!
                        best_unpaid = opt_min(best_unpaid, sd - base + pc)
            done[u] = opt_min(best_unpaid, push[u])

        // State Need: u is allowed to be convinced by parent(u)
        // Either u is unpaid (kids already Done/Push), or u is paid (kids any), or already Done
        let paid_from_parent = opt_add(cost[u], sum_any)
        need[u] = opt_min(done[u], opt_min(sum_done, paid_from_parent))

    opt_min(done[V.first], push[V.first])!

@brute(solve)
fn solve_brute() -> Int
    where n <= 14:
    let g = Graph<V, E>.undirected(edges)
    var best = Int.MAX
    for mask in 0..(1 << n):
        var paid_cost = 0
        var conv: [V]Bool = false
        var queue: List<V> = []
        for v in V:
            if ((mask >> v.to_idx) & 1) == 1: paid_cost += cost[v]
            if init_str[v] == 'Y':
                conv[v] = true
                queue.push(v)
        var head = 0
        while head < queue.len:
            let u = queue[head]
            head += 1
            if ((mask >> u.to_idx) & 1) == 1:
                for a in g.out(u):
                    if not conv[a.to]:
                        conv[a.to] = true
                        queue.push(a.to)
        if all(conv[v] for v in V):
            best min= paid_cost
    best

print(solve())
