// expect: ok, output "-1 YES 1 1 -1 2 0:3 1:4 2:6"
// rule: std.prefix.scan
// A `Prefix` walks the fold through each position, so the running totals take `min`, `any`,
// `inversions` and a comprehension with no loop that pushes them by hand.
let a = [1, -2, 3]
let s = a.scan(+)
domain I = 0..3
let b: [I]Int = [3, 1, 2]
let walked = [t for t in s]
let keyed = ["{v}:{t}" for (v, t) in b.scan(+).items()]
print(s.min()!, yes(s.any(t => t < 0)), inversions(s), walked, keyed)
