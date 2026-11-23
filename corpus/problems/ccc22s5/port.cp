// Runnable port (proto/cp.py lib-run) of statement-first ccc22s5.cp
// original: corpus/submissions/4371349.cpp

let n: Int = read()
domain V = 1..=n
domain E = 0..(n - 1)
let edges: [E](u: V, v: V) = read()
let init_s: Str = read()
let cost: [V]Int = read()

fn opt_add(a: ?Int, b: ?Int) -> ?Int:
    if let x = a:
        if let y = b: x + y
        else: None
    else: None

fn opt_min(a: ?Int, b: ?Int) -> ?Int:
    if let x = a:
        if let y = b: min(x, y)
        else: x
    else: b

let g = Graph<V, E>.undirected(edges)
let tree = Rooted(u => g.out(u), V.first)

var need: [V]?Int = [V] _ => None
var done: [V]?Int = [V] _ => None
var push: [V]?Int = [V] _ => None

for u in tree.at.rev:
    let is_y = init_s[u.to_idx] == 'Y'

    var sum_done: ?Int = 0
    var sum_any: ?Int = 0
    for c in tree.children(u):
        sum_done = opt_add(sum_done, opt_min(done[c], push[c]))
        sum_any = opt_add(sum_any, opt_min(need[c], opt_min(done[c], push[c])))

    if is_y:
        push[u] = opt_add(cost[u], sum_any)
    else:
        var best_push: ?Int = None
        if let sa = sum_any:
            for c in tree.children(u):
                if let pc = push[c]:
                    let base = opt_min(need[c], opt_min(done[c], pc))!
                    best_push = opt_min(best_push, sa - base + pc)
        push[u] = opt_add(cost[u], best_push)

    if is_y:
        done[u] = opt_min(sum_done, push[u])
    else:
        var best_unpaid: ?Int = None
        if let sd = sum_done:
            for c in tree.children(u):
                if let pc = push[c]:
                    let base = opt_min(done[c], pc)!
                    best_unpaid = opt_min(best_unpaid, sd - base + pc)
        done[u] = opt_min(best_unpaid, push[u])

    let paid_from_parent = opt_add(cost[u], sum_any)
    need[u] = opt_min(done[u], opt_min(sum_done, paid_from_parent))

print(opt_min(done[V.first!], push[V.first!])!)
