// expect: ok, output "12 8 6 bc ac ab"
// Acceptance for seq/U38 (fold of everything but one position), handbook/product/std/sequences-usecases.md: it is `op(pre[i.l], suf[i.r])` over `a.scan(op)` and `a.rev.scan(op)`, with no `prefix_excl` (decision 511). A non-commutative op keeps the order: `pre[i.l]` folds left of `i`, `suf[i.r]` folds right of it, each left to right.
domain D = 0..3
let a: [D]Int = [2, 3, 4]
let pre = a.scan(*)
let suf = a.rev.scan(*)
let s: [D]Str = ["a", "b", "c"]
let m = Monoid("", (x: Str, y: Str) => x + y)
let spre = s.scan(m)
let ssuf = s.rev.scan(m)
print([D] i => pre[i.l] * suf[i.r], [D] i => spre[i.l] + ssuf[i.r])
