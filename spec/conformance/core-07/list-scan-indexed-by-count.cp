// expect: ok, output "0 4 8"
// rule: 07.4.1
// A `List`'s cuts are `Int`s: `pre[k]` folds the first `k`.
let pre = [3, 1, 4].scan(+)
print(pre[0], pre[2], pre[3])
