// CMT 2021 R2 (ztr), tuned: port.cp's 2-SAT with its implication graph built straight into a
// `Graph`, whose arcs `scc` walks as slices; `TwoSat.solve` hands `scc` a closure that
// copies each node's list. Same clauses in the same order, so the same answer.
enum Side: Below | Above
let n: 1..=1e5 = read()
let m: 1..=1e5 = read()
domain Slot = 1..=n
let facts: [m](i: Slot, j: Slot, k: Slot, x: 1..=1e9) = read()
let (domain Pin, pins) = compress([(s:, x: f.x) for f in facts for s in [f.i, f.j, f.k]])
// Question 2p + side asks whether pin p's number is below (above) its value; literal node
// 2q + 1 says question q holds, 2q that it fails.
domain Node = 0..4 * Pin.count
fn lit(p: Pin, side: Side, value: Bool) -> Int = 2 * (2 * p.ord + side.ord) + Int(value)
fn either(mut arcs: List<(from: Node, arc: Node)>, a: Int, b: Int):
    arcs.push((from: Node(a ^ 1), arc: Node(b)))
    arcs.push((from: Node(b ^ 1), arc: Node(a)))

var arcs: List<(from: Node, arc: Node)> = []
for f in facts:
    let ps = [pins.index_of((s:, x: f.x)) for s in [f.i, f.j, f.k]]
    for side in Side:
        for (a, b) in ps.pairs(): either(mut arcs, lit(a, side, false), lit(b, side, false))
for p in Pin: either(mut arcs, lit(p, Below, false), lit(p, Above, false))
for (p, q) in Pin.collect().adjacent if pins.get(p).s == pins.get(q).s:
    either(mut arcs, lit(p, Above, true), lit(q, Below, true))
    either(mut arcs, lit(p, Above, false), lit(q, Below, false))

let (C, comp) = scc(Graph.of(arcs))
fn holds(p: Pin, side: Side) -> Bool = comp[Node(lit(p, side, true))] > comp[Node(lit(p, side, false))]
if Node.all(v => v.ord % 2 == 1 or comp[v] != comp[Node(v.ord + 1)]):
    print("YES")
    var number: [Slot]Int = 1
    for p in Pin:
        let pin = pins.get(p)
        if holds(p, Above): number[pin.s] max= pin.x + 1
        elif not holds(p, Below): number[pin.s] max= pin.x
    print(number)
else:
    print("NO")
