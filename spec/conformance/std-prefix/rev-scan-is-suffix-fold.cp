// expect: ok, output "14 9 8 4 3 14 5 4 4 3 3"
// gap: compiler 2026-11-24T05Z
// 07-sequences.md, scans ("suf[c]: fold of everything right of cut c"): `a.rev.scan(op)` is the
// suffix fold, keyed by the source's own cuts, so `suf.cut(i)` folds `a[i..]` (on Int keys a
// cut is read by count, decision 507).
let a = [5, 1, 4, 1, 3]
let s = a.rev.scan(+)
let m = a.rev.scan(max)
print([s.cut(i) for i in a.keys], s.all, [m.cut(i) for i in a.keys])
