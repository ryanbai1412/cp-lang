// expect: ok
// rule: 05.7.6
var a = List(1, 2)
var cnt = List(0, 0, 0, 0)
let i = 0
a.push(a.len)
cnt[a[i]] += 1
