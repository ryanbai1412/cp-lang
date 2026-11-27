// expect: ok
// rule: 05.7.6
var a = [1, 2]
var cnt = [0, 0, 0, 0]
let i = 0
a.push(a.len)
cnt[a[i]] += 1
