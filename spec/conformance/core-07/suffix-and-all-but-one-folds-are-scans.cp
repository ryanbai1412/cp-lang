// expect: ok, output "9 7 4 0\n12 8 6"
// rule: 07.4.16
// A suffix fold is `a.rev.scan(op)` on the same cuts, and all but `i` is `op(pre[i.l], suf[i.r])`; std has no `suffix_fold` or `prefix_excl` (decision 511).
domain D = 0..3
let a: [D]Int = [2, 3, 4]
let suf = a.rev.scan(+)
print(suf.cut(0), suf.cut(1), suf.cut(2), suf.cut(3))
let pre = a.scan(*)
let sp = a.rev.scan(*)
let out: [D]Int = [D] i => pre[i.l] * sp[i.r]
print(out[D(0)], out[D(1)], out[D(2)])
