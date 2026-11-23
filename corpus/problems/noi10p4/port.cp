// NOI 2010 Flight Control: n planes take off one per time slot 1..n. Plane
// i must leave no later than slot k[i], and m rules say plane a leaves
// before plane b. A valid schedule exists. Print one, then for every plane
// the earliest slot it can have in any valid schedule.
//
// Push the deadlines back along the rules: a plane must leave before the
// tightened deadline of everything after it. Sorting by tightened deadline
// is then a schedule. For plane u, send every plane that need not precede u
// as late as its deadline allows; u and its predecessors get the slots
// left over, and u takes the last of them.
// original: corpus/submissions/5670273.cpp
use std.graph.{topo}

let (n, m): (Int, Int) = read()
domain Plane = 1..=n
domain Slot = 1..=n
let k: [Plane]Int = read()
var after: [Plane]List<(to: Plane)> = []
var before: [Plane]List<Plane> = []
for _ in 0..m:
    let (a, b): (Int, Int) = read()
    after[Plane(a)].push((to: Plane(b)))
    before[Plane(b)].push(Plane(a))

// One line of numbers. The interpreter has no `" ".join(...)`.
fn line(xs: List<Int>) -> Str:
    var out = Str()
    for x in xs:
        if not out.is_empty: out.push(' ')
        for c in "{x}": out.push(c)
    out

var due = [Plane] p => k[p]
let order = topo::<Plane>(p => after[p])!
for p in order.rev:
    for a in after[p]: due[p] = min(due[p], due[a.to] - 1)

print(line([p.to_idx + 1 for p in sorted([p for p in Plane], by: p => due[p])]))

fn earliest(u: Plane) -> Int:
    var first: [Plane]Bool = false  // u and everything that must precede it
    var stack = [u]
    first[u] = true
    while not stack.is_empty:
        let p = stack.pop()!
        for b in before[p]:
            if not first[b]:
                first[b] = true
                stack.push(b)
    var due_at: [Slot]Int = 0
    for p in Plane:
        if not first[p]: due_at[Slot(due[p])] += 1
    var waiting = 0
    for slot in Slot.rev:
        waiting += due_at[slot]
        if waiting == 0: return slot.to_idx + 1
        waiting -= 1
    assert false
    0

print(line([earliest(u) for u in Plane]))
