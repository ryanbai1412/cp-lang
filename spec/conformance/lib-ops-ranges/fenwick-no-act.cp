// expect: E0705
let a = [1, 2, 3]
var f = Fenwick(+, a, act: add) // <- here
