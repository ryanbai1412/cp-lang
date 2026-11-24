// expect: ok, output "ok"
// input: "1 300\n"
// model: MergeSegs
// A MergeSegs pool of sets against bitmasks: seeded random singleton, merge (both inputs stay
// valid) and kills of positions, with first_alive checked exactly and first_ge, any_in checked
// against what pruning may have removed. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.segtree.{MergeSeg, MergeSegs}
let seed0 = read<Int>()
let n = read<Int>()
domain D = 0..20
var rng = Rng(seed0)
var p = MergeSegs<D>()
var sets: List<MergeSeg<D>> = []
var masks: List<Int> = []
var dead = [false for _ in 0..20]
fn has(m: Int, x: Int) -> Bool = (m >> x) & 1 == 1
var bad = ""
for k in 0..n:
    let op = rng.int(0, 5)
    let x = rng.int(0, 19)
    if op == 0 or sets.len == 0:
        sets.push(p.singleton(D(x)))
        masks.push(1 << x)
    elif op == 1:
        let (i, j) = (rng.int(0, sets.len - 1), rng.int(0, sets.len - 1))
        sets.push(p.merge(sets[i], sets[j]))
        masks.push(masks[i] | masks[j])
    elif op == 2:
        dead[x] = true
    elif op == 3:
        let i = rng.int(0, sets.len - 1)
        var s = sets[i]
        let got = p.first_alive(mut s, y => not dead[y.ord])
        sets[i] = s
        let want = (0..20).find(y => has(masks[i], y) and not dead[y])
        if got.map(y => y.ord) != want: bad = "op {k}: first_alive set {i}"
    elif op == 4:
        let i = rng.int(0, sets.len - 1)
        let alive = (x..20).find(y => has(masks[i], y) and not dead[y])
        match p.first_ge(sets[i], D(x)):
            None:
                if alive != None: bad = "op {k}: first_ge {x} is None"
            Some(g):
                let q = g.ord
                if q < x or not has(masks[i], q) or (x..q).any(y => has(masks[i], y) and not dead[y]) or (not dead[q] and alive != Some(q)):
                    bad = "op {k}: first_ge {x} gave {q}"
    else:
        let i = rng.int(0, sets.len - 1)
        let hi = rng.int(x, 20)
        let got = p.any_in(sets[i], D.cuts.nth(x)..D.cuts.nth(hi))
        if (x..hi).any(y => has(masks[i], y) and not dead[y]) and not got: bad = "op {k}: any_in {x}..{hi} false"
        if not (x..hi).any(y => has(masks[i], y)) and got: bad = "op {k}: any_in {x}..{hi} true"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
