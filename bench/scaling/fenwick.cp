use std.fenwick.Fenwick
let n: Int = read()
domain V = 0..n
var s = 0
let a: [V]Int = [V] v => Int(v) % 5 + 1

for _ in 0..400:
    let f = Fenwick<V, Int>(a) // timed: fenwick.Fenwick.new
    s += f.all
print(s)
