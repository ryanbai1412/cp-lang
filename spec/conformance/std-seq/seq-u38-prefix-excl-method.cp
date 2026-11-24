// expect: ok, output "12 8 6"
// Acceptance for seq/U38 (fold of everything but one position), handbook/product/std/sequences-usecases.md: it is `op(pre[i.l], suf[i.r])` over `a.scan(op)` and `a.rev.scan(op)`, with no `prefix_excl` (decision 511).
domain D = 0..3
let a: [D]Int = [2, 3, 4]
let pre = a.scan(*)
let suf = a.rev.scan(*)
print([D] i => pre[i.l] * suf[i.r])
