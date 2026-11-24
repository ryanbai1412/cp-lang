// expect: ok, output "0 4 8"
// rule: 07.4.1 07.1.6
// A `List`'s cuts are `Int`s, so a count reads its scan by name: `pre.cut(k)` folds the first `k` (decision 507).
let pre = [3, 1, 4].scan(+)
print(pre.cut(0), pre.cut(2), pre.cut(3))
