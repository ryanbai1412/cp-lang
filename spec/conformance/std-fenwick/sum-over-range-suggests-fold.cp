// expect: E0203
// has: "combines a range with `fold`"
var f = Fenwick<0..4>()
let l = 1
print(f.sum(l..=l)) // <- here
