// expect: E0320
// rule: 07.1.6
// On a `List` scan a position and a cut are both `Int`, so `pre[k]` can't say which it means; the help is `pre.cut(k)` (decision 507).
let pre = scan([3, 1, 4], +)
print(pre[2]) // <- here
