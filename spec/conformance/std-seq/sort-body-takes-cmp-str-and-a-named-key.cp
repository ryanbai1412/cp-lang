// expect: ok, output "3 2 1 | 3 2 1 | abc | cba | 3 2 1 | 0 1 1 zba"
// rule: std.seq.sort
// std's own `sort`/`sorted` body (called qualified, so no builtin answers) takes `cmp:`, a `Str`, and a fn named as the key.
use std.seq
var xs = [1, 3, 2]
seq.sort(mut xs, cmp: (a, b) => a > b)
let ys = seq.sorted([2, 1, 3], cmp: (a, b) => a > b)
var s = "cab"
seq.sort(mut s)
let t = seq.sorted("bca", cmp: (a, b) => a > b)
var zs = [2, 1, 3]
seq.sort(mut zs, by: desc)
var ps = [(1, 'b'), (0, 'z'), (1, 'a')]
seq.sort(mut ps, cmp: (a, b) => a.0 < b.0)
print(xs, "|", ys, "|", s, "|", t, "|", zs, "|", [p.0 for p in ps], [p.1 for p in ps])
