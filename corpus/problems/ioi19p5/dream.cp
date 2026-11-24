// IOI 2019 "Vision Program": build an AND/OR/NOT circuit over an h x w image
// with exactly two black pixels that outputs whether they are at distance k.
// Two pixels are at distance exactly k when some pair of diagonals k apart
// (in either direction) both hold a black pixel and no pair k + 1 apart does.
//
// This plays the sample grader: read `H W K`, build the network once, then
// read images `r1 c1 r2 c2` until -1 and print the network's output on each.
enum Op = And | Or | Not

let (h, w) = read<(1..=200, 1..=200)>()
let k = read<1..=h + w - 2>()
domain R = [h]
domain C = [w]
fn wire(r: R, c: C) -> Int = Int(r) * w + Int(c)

var gates: List<(op: Op, ins: List<Int>)> = []
var fan_in = 0
fn add(op: Op, ins: List<Int>) -> Int:
    let id = h * w + gates.len
    assert not ins.is_empty and ins.all(i => 0 <= i and i < id), "inputs are earlier wires"
    fan_in += ins.len
    assert gates.len < 10_000 and fan_in <= 1_000_000, "within the instruction limits"
    gates.push((op:, ins:))
    id
fn add_and(ins: List<Int>) -> Int = add(And, ins)
fn add_or(ins: List<Int>) -> Int = add(Or, ins)
fn add_not(x: Int) -> Int = add(Not, [x])

fn construct_network():
    fn any(ws: List<Int>) -> Int = if ws.is_empty: add_and([0, add_not(0)]) else: add_or(ws)

    // one OR wire per diagonal, in diagonal order
    fn lines(key: (R, C) -> Int) -> List<Int>:
        let by_line = [(r, c) for r in R for c in C].group(by: rc => key(rc.0, rc.1), of: rc => wire(rc.0, rc.1))
        [add_or(by_line[d]) for d in by_line.keys.collect().sorted()]

    fn gap_exactly(ls: List<Int>, d: Int) -> Int =
        any([add_and([ls[i], ls[i + d]]) for i in 0..ls.len - d])

    // seen[i]: some line at or before i is lit
    fn gap_at_least(ls: List<Int>, d: Int) -> Int:
        var seen: List<Int> = []
        for l in ls: seen.push(if let s = seen.last: add_or([s, l]) else: l)
        any([add_and([seen[i], ls[i + d]]) for i in 0..ls.len - d])

    let diag = [lines((r, c) => Int(r) + Int(c)), lines((r, c) => Int(r) - Int(c))]
    let hit = add_or(diag.map(ls => gap_exactly(ls, k)).collect())
    let over = add_or(diag.map(ls => gap_at_least(ls, k + 1)).collect())
    add_and([hit, add_not(over)])

construct_network()
while true:
    let r1 = read<-1..h>()
    if r1 == -1: break
    let (c1, r2, c2) = read<(C, R, C)>()
    var on = List.filled(h * w, false)
    on[wire(R(r1), c1)] = true
    on[wire(r2, c2)] = true
    for g in gates:
        on.push(match g.op:
            And: g.ins.all(i => on[i])
            Or: g.ins.any(i => on[i])
            Not: not on[g.ins[0]])
    print(Int(on.last!))
