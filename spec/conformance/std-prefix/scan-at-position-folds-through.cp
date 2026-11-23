// expect: ok, output "1 3 6"
// rule: 07.1.6
// `pre[v]` at a position folds through `v`, not just `a[v]` (rebuild 8, H-INT-KEY-PREFIX).
domain I = 0..3
let a: [I]1..=9 = [I] i => Int(i) + 1
let pre = a.scan(+)
print(pre[I(0)], pre[I(1)], pre[I(2)])
