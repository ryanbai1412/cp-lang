// Source: NOI 2007 Necklace Factory, https://dmoj.ca/problem/noi07p4
// limits: 1000 ms, 256 MB
// Problem: a necklace of n <= 5e5 beads with colours <= 1000 at fixed positions 1..=n clockwise,
//   and q <= 5e5 commands: R k rotates the beads k places clockwise, F reflects them in the axis
//   through position 1, S i j swaps two beads, P i j x paints positions i..=j clockwise colour x,
//   C prints the number of same-colour parts round the whole necklace, CS i j that of the segment
//   i..=j clockwise (still a segment when it has all n beads).
// Solution: beads stay where they are in a lazy segment tree over physical slots, nodes holding
//   (left colour, right colour, parts) and paint as the action. Rotation and flip only change the
//   map from position to slot, x -> (dir * x + off) mod n. A segment maps to a slot range,
//   (left colour, right colour, parts) and paint as the action. Rotation and flip only change the
//   map from position to slot, x -> (dir * x + off) mod n (`Slot.wrap`). A segment maps to one
//   slot range, or two when it wraps; C merges the ends if they match. O((n + q) log n).
// original: corpus/submissions/5032746.cpp
// Bounds: n: 1..=5e5; colours: 1..=1000; bead: 1..=colours; q: 1..=5e5; x: 1..=colours
let (n, colours): (1..=5e5, 1..=1000) = read()
domain Pos = 1..=n
domain Slot = 0..n
let bead: [Slot]1..=colours = read()
let q: 1..=5e5 = read()

struct Run(l: Int, r: Int, parts: Int):
    fn of(c: Int) -> Run = Run(c, c, 1)

let joined = ((a: Run, b: Run) => Run(a.l, b.r, a.parts + b.parts - Int(a.r == b.l))).opt()
let paint = Action<?Run, ?Int>(
    id: None,
    compose: (f, g) => if f == None: g else: f,
    apply: (f, x, len) => if let c = f: x.map(_ => Run.of(c)) else: x,
    over: joined,
)
var beads = SegTree(joined, [Slot] s => Some(Run.of(bead[s])), act: paint)

var (dir, off) = (1, 0)
fn slot(p: Pos) -> Slot = Slot.wrap(dir * (Int(p) - 1) + off)

// the slots of positions i..=j clockwise: one range, or two when it wraps
fn arcs(i: Pos, j: Pos) -> List<Range<Slot>>:
    var (l, r) = (slot(i), slot(j))
    if dir == -1: (l, r) = (r, l)
    if l <= r: [l..=r] else: [l..=Slot.last!, Slot.first!..=r]

for _ in 0..q:
    let op: Str = read()
    if op == "R":
        let k: Int = read()
        off -= dir * k
    elif op == "F":
        dir = -dir
    elif op == "S":
        let (i, j): (Pos, Pos) = read()
        let (a, b) = (slot(i), slot(j))
        let (ca, cb) = (beads.get(a)!.l, beads.get(b)!.l)
        beads.set(a, Some(Run.of(cb)))
        beads.set(b, Some(Run.of(ca)))
    elif op == "P":
        let (i, j, x): (Pos, Pos, 1..=colours) = read()
        for a in arcs(i, j): beads.apply(a, x)
    elif op == "C":
        let all = beads.all!
        print(all.parts - Int(all.parts > 1 and all.l == all.r))
    else:
        let (i, j): (Pos, Pos) = read()
        var runs: ?Run = None
        for a in arcs(i, j): runs = joined.op(runs, beads.fold(a))
        print(runs!.parts)
