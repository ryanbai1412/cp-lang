// CCC 2010 S4, Animal Farm: pens are polygons given by their corners, and
// every wall has a cost to knock down. A wall between two pens joins them;
// a wall of one pen opens it to the outside. Print the cheapest way to let
// all the animals meet, inside or out.
// original: corpus/submissions/7171512.cpp
use std.dsu.{Dsu}

let pens: 1..=100 = read()
domain Pen = 1..=pens
domain Yard = 0..=pens                                    // pen 0 is the outside
domain Corner = 1..=1000

// Each wall by its two corners, smaller first: its cost and the pens beside it.
var cost_of = Map<(Corner, Corner), Int>()
var beside = Map<(Corner, Corner), List<Yard>>()
for p in Pen:
    let sides: 3..=8 = read()
    let corner: [sides]Corner = read()
    let cost: [sides]0..=5000 = read()
    for j in 0..sides:                                    // WISH L18: for (j, a, b) in corner.cyc.pairs()
        let a = corner[j]
        let b = corner[(j + 1) % sides]
        let wall = (min(a, b), max(a, b))
        cost_of[wall] = cost[j]
        if wall not in beside: beside[wall] = []          // WISH L17: beside[wall].push(...) on a key that is new
        beside[wall].push(Yard(p.label))

var doors: List<(Int, Yard, Yard)> = []
for (wall, who) in beside:
    if who.len == 2: doors.push((cost_of[wall], who[0], who[1]))
    else: doors.push((cost_of[wall], who[0], Yard(0)))
doors.sort()

// The cheapest set of doors that joins every pen, with or without going outside.
fn cheapest(outside: Bool) -> ?Int:                       // WISH L19: mst(doors) -> ?(cost, edges), none when it cannot span
    var joined = Dsu<Yard>()
    var total = 0
    var used = 0
    for (cost, a, b) in doors:
        if (outside or b != Yard(0)) and joined.unite(a, b):
            total += cost
            used += 1
    let need = if outside: pens else: pens - 1
    if used == need: return total
    return none

print(min(cheapest(false), cheapest(true)!))
