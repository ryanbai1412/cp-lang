// CCO '07 P5 — UFO / Moving Target on a Ring (Interactive) (DMOJ: cco07p5)
// original: corpus/submissions/5120565.cpp
// CP Lang 0.2.2 (locked spec, docs/one-page-cheat-sheet.md)
//
// Interactive problem: an object on a ring 0..n starts at unknown position p
// with constant velocity v in 0..n. Each "Look l r" query returns "True"/"False"
// indicating whether p is in [l, r], then advances p = (p + v) % n.
// Probe random 2-cell windows until a hit, then halve the candidate (p, v) set
// by median position until a single state remains and output "Guess v p".
let n: 2..=1000 = ask("GetSize")
domain Pos = 0..n
const BLOCK = 2

fn look(l: Int, r: Int) -> Bool:
    let tok: Str = ask("Look", l, r)
    tok == "True"

var x = 0
while true:
    x = rand(0, n - BLOCK)
    if look(x, x + BLOCK - 1): break

var cands: List<(p: Pos, v: Int)> = []
for v in 0..n:
    for j in 0..BLOCK:
        cands.push((Pos.first.cyc(x + v + j), v))

while cands.len > 1:
    cands.sort()
    let mid = cands[(cands.len - 1) / 2].p
    let hit = look(0, mid.to_idx)
    cands = [
        (c.p.cyc(c.v), c.v)
        for c in cands
        if (hit and c.p <= mid) or (not hit and c.p > mid)
    ]

let (p, v) = cands[0]
print("Guess", v, p)

@interactor
fn judge(mut sol: Interactor):
    let hidden_n: 2..=1000 = read()
    var p: 0..hidden_n = read()
    let v: 0..hidden_n = read()
    while true:
        let cmd: Str = sol.recv()
        if cmd == "GetSize": sol.send(hidden_n)
        elif cmd == "Look":
            let (l, r): (Int, Int) = sol.recv()
            sol.send(if p in l..=r: "True" else: "False")
            p = (p + v) % hidden_n
        elif cmd == "Guess":
            let (gv, gp): (Int, Int) = sol.recv()
            assert gv == v and gp == p
            return
