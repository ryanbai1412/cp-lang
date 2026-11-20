// CCC 2017 S4 Minimum Cost Flow: the first n - 1 pipes are the current plan,
// and one pipe a day may be swapped for another. An enhancer then takes up
// to d off the cost of one pipe. Print the fewest days to reach a cheapest
// plan.
//
// Kruskal, with old pipes ahead of new ones of the same cost, keeps as much
// of the current plan as a cheapest plan can. The enhancer saves one more
// day when the dearest pipe of the plan can be traded for an old pipe that
// the enhancer makes free.

let (n, m, d): (Int, Int, Int) = read()
domain House = 1..=n
domain Pipe = 0..m
let pipes: [Pipe](u: House, v: House, cost: Int) = read()
fn is_old(p: Pipe) -> Bool = p.to_idx < n - 1
let cheapest_first = sorted(Pipe, by: p => (pipes[p].cost, not is_old(p)))

var plan = Dsu<House>()
var chosen: [Pipe]Bool = false
var dearest = 0
for p in cheapest_first:
    if plan.unite(pipes[p].u, pipes[p].v):
        chosen[p] = true
        dearest = pipes[p].cost
var days = sum(1 for p in Pipe if chosen[p] and not is_old(p))

if dearest <= d:
    // the plan without its new dearest pipes; an old pipe that the enhancer
    // makes free and that still joins two parts replaces one of them
    var rest = Dsu<House>()
    for p in Pipe:
        if chosen[p] and (is_old(p) or pipes[p].cost < dearest): rest.unite(pipes[p].u, pipes[p].v)
    for p in Pipe:
        if not chosen[p] and is_old(p) and pipes[p].cost <= d and not rest.same(pipes[p].u, pipes[p].v):
            days -= 1
            break
print(days)
