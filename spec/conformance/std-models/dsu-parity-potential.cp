// expect: ok, output "ok"
// input: "1 300\n"
// model: ParityDsu
// model: PotentialDsu
// ParityDsu<V> and PotentialDsu<V, Int> against a root and an offset per position, relabelled on
// each merge: seeded random unite (agreeing, implied and contradicting), mark and undo_to, with
// find, diff, same, size, components, groups and consistent compared after each op.
// Reads `seed n`; prints the first op that differs.
use std.dsu.{Mark}
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..7
var rng = Rng(seed0)
var p = ParityDsu<V>()
var q = PotentialDsu<V, Int>()
var proot = [i for i in 0..7]
var side = List.filled(7, false)
var pbad = 0
var qroot = [i for i in 0..7]
var pot = List.filled(7, 0)
var qbad = 0
var marks: List<(Mark, Mark, List<Int>, List<Bool>, Int, List<Int>, List<Int>, Int)> = []
var bad = ""
for k in 0..n:
    let op = rng.int(0, 4)
    let (a, b) = (rng.int(0, 6), rng.int(0, 6))
    if op == 0:
        let differ = rng.int(0, 1) == 1
        let got = p.unite(V(a), V(b), differ:)
        let (ra, rb) = (proot[a], proot[b])
        var want = true
        if ra == rb:
            want = (side[a] != side[b]) == differ
            if not want: pbad += 1
        else:
            let (sa, sb) = (proot.count(r => r == ra), proot.count(r => r == rb))
            let (kept, gone) = if sb > sa: (rb, ra) else: (ra, rb)
            let flip = (side[a] != side[b]) != differ
            for i in 0..7:
                if proot[i] == gone:
                    proot[i] = kept
                    side[i] = side[i] != flip
        if got != want: bad = "op {k}: parity unite {a} {b} {yes(differ)}"
    elif op == 1:
        let d = rng.int(-3, 3)
        let got = q.unite(V(a), V(b), d)
        let (ra, rb) = (qroot[a], qroot[b])
        var want = true
        if ra == rb:
            want = pot[b] - pot[a] == d
            if not want: qbad += 1
        else:
            let (sa, sb) = (qroot.count(r => r == ra), qroot.count(r => r == rb))
            let (kept, gone) = if sb > sa: (rb, ra) else: (ra, rb)
            let shift = if gone == rb: d + pot[a] - pot[b] else: pot[b] - pot[a] - d
            for i in 0..7:
                if qroot[i] == gone:
                    qroot[i] = kept
                    pot[i] += shift
        if got != want: bad = "op {k}: potential unite {a} {b} {d}"
    elif op == 2:
        marks.push((p.mark(), q.mark(), proot, side, pbad, qroot, pot, qbad))
    elif op == 3 and marks.len > 0:
        let i = rng.int(0, marks.len - 1)
        let (mp, mq, r0, s0, b0, r1, o1, b1) = marks[i]
        p.undo_to(mp)
        q.undo_to(mq)
        (proot, side, pbad, qroot, pot, qbad) = (r0, s0, b0, r1, o1, b1)
        while marks.len > i + 1: marks.pop()
    if bad != "": break
    if p.consistent != (pbad == 0) or q.consistent != (qbad == 0):
        bad = "op {k}: consistent"
    elif p.components != [i for i in 0..7 if proot[i] == i].len:
        bad = "op {k}: parity components"
    elif q.components != [i for i in 0..7 if qroot[i] == i].len:
        bad = "op {k}: potential components"
    elif p.groups() != [[V(i) for i in 0..7 if proot[i] == r] for r in sorted([proot[i] for i in 0..7 if proot[i] == i], by: r => [i for i in 0..7 if proot[i] == r][0])]:
        bad = "op {k}: parity groups"
    for i in 0..7:
        if p.find(V(i)) != (root: V(proot[i]), flip: side[i]) or p.size(V(i)) != proot.count(r => r == proot[i]):
            bad = "op {k}: parity find or size {i}"
        if q.find(V(i)) != (root: V(qroot[i]), offset: pot[i]) or q.size(V(i)) != qroot.count(r => r == qroot[i]):
            bad = "op {k}: potential find or size {i}"
        for j in 0..7:
            let pd = if proot[i] == proot[j]: Some(side[i] != side[j]) else: None
            let qd = if qroot[i] == qroot[j]: Some(pot[j] - pot[i]) else: None
            if p.diff(V(i), V(j)) != pd or p.same(V(i), V(j)) != (pd != None):
                bad = "op {k}: parity diff {i} {j}"
            if q.diff(V(i), V(j)) != qd or q.same(V(i), V(j)) != (qd != None):
                bad = "op {k}: potential diff {i} {j}"
    if bad != "": break
print(if bad == "": "ok" else: bad)
