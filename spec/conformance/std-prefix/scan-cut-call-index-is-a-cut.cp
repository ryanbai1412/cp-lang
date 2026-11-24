// expect: ok, output "2 5"
// rule: 07.1.6
// An index made by `cut_before`/`cut_after` already says it is a cut, so it isn't E0320 (decision 507).
let a = [2, 3, 4]
let pre = a.scan(+)
print(pre[a.cut_before(1)], pre[a.cut_after(1)])
