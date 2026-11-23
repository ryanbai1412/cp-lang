// expect: ok, output "YES\nYES\nYES\nYES\nYES YES\nYES\nYES YES\nYES"
use std.beats.{HistoricBeatsSegTree}
use std.rand.Rng
domain E = 0..2
let b12: [E]Int = [1, 2]
let b51: [E]Int = [5, 1]
let all = E(0)..=E(1)
let first = E(0)..=E(0)
let second = E(1)..=E(1)
var t0 = HistoricBeatsSegTree(b12); print(yes(t0.hist_sum(all) == 3))
var t1 = HistoricBeatsSegTree(b51); t1.chmin(all, 2); print(yes(t1.sum(all) == 3 and t1.hist_max(first) == Some(5)))
var t2 = HistoricBeatsSegTree(b51); t2.chmax(all, 3); print(yes(t2.sum(all) == 8))
var t3 = HistoricBeatsSegTree(b12); t3.add(first, 3); print(yes(t3.sum(all) == 6))
var t4 = HistoricBeatsSegTree(b12); t4.add(all, 5); t4.add(all, -10); print(yes(t4.max_val(first) == Some(-4)), yes(t4.hist_max(first) == Some(6)))
var t9 = HistoricBeatsSegTree(b12); print(yes(t9.hist_max(E(1).l..E(1).l) == None))
var t6 = HistoricBeatsSegTree(b12); t6.add(first, 3); print(yes(t6.hist_sum(all) == 9), yes(t6.hist_sum(second) == 4))
domain W = 0..12
var rng = Rng(3)
fn r(mut g: Rng, m: Int) -> Int = Int(g.next() % U64(m))
fn cut(i: Int) -> Cut<W> = if i < 12: W(i).l else: W(11).r
var a: [W]Int = 0
for i in W: a[i] = r(mut rng, 21) - 10
var t = HistoricBeatsSegTree<W>.new(a: a)
var cur = [a[i] for i in W]
var hmx = cur.clone()
var hs = cur.clone()
var ok = true
for step in 0..3000:
    let op = r(mut rng, 6)
    let l = r(mut rng, 12)
    let h = l + r(mut rng, 13 - l)
    let x = r(mut rng, 21) - 10
    let rg = span(cut(l), cut(h))
    if op <= 2:
        if op == 0: t.chmin(rg, x)
        elif op == 1: t.chmax(rg, x)
        else: t.add(rg, x)
        for i in l..h:
            if op == 0: cur[i] min= x
            elif op == 1: cur[i] max= x
            else: cur[i] += x
        for i in 0..12:
            hmx[i] = max(hmx[i], cur[i])
            hs[i] += cur[i]
    else:
        var (sm, mv, hm, hsum) = (0, Int.MIN, Int.MIN, 0)
        for i in l..h:
            sm += cur[i]
            mv = max(mv, cur[i])
            hm = max(hm, hmx[i])
            hsum += hs[i]
        if t.sum(rg) != sm or t.hist_sum(rg) != hsum: ok = false
        if h > l and (t.max_val(rg) != Some(mv) or t.hist_max(rg) != Some(hm)): ok = false
        if h == l and (t.max_val(rg) != None or t.hist_max(rg) != None): ok = false
        if not ok:
            print("bad at", step, op, l, h)
            break
print(yes(ok))
