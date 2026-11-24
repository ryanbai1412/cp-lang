// expect: ok, output "0 1 6\n3"
// rule: 07.1.6
// `pre.cut(k)` folds the first `k`, with `n + 1` entries; a named domain whose base is not 0 counts
// from its first position, so `cut(2)` there is still "the first two" (decision 507).
let p = [1, 2, 3].scan(+)
print(p.cut(0), p.cut(1), p.cut(3))
domain D = 5..=7
let q: [D]Int = [1, 2, 9]
print(q.scan(+).cut(2))
