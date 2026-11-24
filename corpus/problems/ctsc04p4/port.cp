// Source: CTSC 2004 (Network Upgrade), https://dmoj.ca/problem/ctsc04p4
// Problem: a tree of n <= 400 switches with link lengths below 20000 and upgrade costs c_i.
//   Upgrade 1 to p switches to cores; the cost is the sum of their c_i plus, for every other
//   switch, the distance to its nearest core. Print the least cost, then the fewest cores that
//   reach it.
// Solution: plan(u)[v] is the staircase (more extra cores, strictly lower cost) for u's subtree
//   when u's nearest core is v, opened for free, or a closer core. Children (largest first)
//   merge knapsack-style, capped by the cores the subtree can hold; then, scanning the cores a
//   by distance from u, u either uses a or a closer core opened at c_a. The answer is the best
//   last step of plan(root)[v] plus c_v. About O(n^2 p).
// original: corpus/submissions/4529641.cpp
let n: 1..=400 = read()
let p: 1..=1e9 = read()
domain Switch = 1..=n
domain Link = 1..n
let cost: [Switch]0..=1e9 = read()
let links: [Link](u: Switch, v: Switch, len: 0..20_000) = read()
let tree = Rooted(Graph.undirected(links).weighted(a => links[a.e].len), Switch.first!)
let dist: [Switch, Switch]Int = [Switch, Switch] (a, b) => tree.dist(a, b)
let extra = min(p - 1, n)
type Steps = List<(cores: Int, cost: Int)>

// best[j]: least cost with exactly j extra cores, kept where it changes.
fn staircase(best: [0..=extra]?Int) -> Steps:
    var steps: Steps = []
    for j in 0..=extra:
        if let c = best[j] and steps.last.map(s => s.cost) != c: steps.push((j, c))
    return steps

fn plan(u: Switch) -> [Switch]Steps:
    let kids = tree.children(u).sorted(by: c => desc(tree.size(c)))
    let kid_plans = [(c, plan(c)) for c in kids]
    let alone: [Switch]Steps = [Switch] v =>
        var best: [0..=extra]?Int = None
        best[0] = dist[u, v]
        var held = 1
        for (c, kid) in kid_plans:
            for j in (0..=min(extra, held)).rev:
                let here = best[j]
                best[j] = None
                if let h = here:
                    for s in kid[v]:
                        if j + s.cores > extra: break
                        best[j + s.cores] min= h + s.cost
            held += tree.size(c)
        staircase(best)
    var out: [Switch]Steps = []
    var closer: [0..=extra]?Int = None
    for a in Switch.sort(by: a => dist[u, a]):
        var at_a: [0..=extra]?Int = None
        for s in alone[a]: at_a[s.cores] = s.cost
        for j in 1..=extra: at_a[j] min= at_a[j - 1]
        out[a] = staircase([0..=extra] j => min(closer[j], at_a[j]))
        for j in 1..=extra:
            if let c = at_a[j - 1]: closer[j] min= c + cost[a]
    return out

let root_plan = plan(tree.root)
fn finish(v: Switch) -> (Int, Int):
    let s = root_plan[v].last!
    return (s.cost + cost[v], s.cores + 1)
let (total, cores) = min(finish(v) for v in Switch)!
print(total)
print(cores)
