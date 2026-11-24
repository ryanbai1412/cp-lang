// n: 512
// reserve moves the elements already there. At n = 512 the copy and the doubled block
// stay in cache at 4n too; at n = 16384 the 4n block leaves L2, and the copy grew 9x.
let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
var s = 0
for _ in 0..40000:
    var g = xs.clone()
    g.reserve(2 * n) // timed: builtin.List.reserve
    s += g.len
print(s)
