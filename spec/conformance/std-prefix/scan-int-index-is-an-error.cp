// expect: E0320
// rule: 07.1.6
// The method form types as the free form: `[3, 1, 4].scan(+)` is a scan over `Int` keys, so `pre[k]` is E0320 there too (decision 507).
let pre = [3, 1, 4].scan(+)
print(pre[2]) // <- here
