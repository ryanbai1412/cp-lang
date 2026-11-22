let n: 1..=9 = read()
domain I = 1..=n
let a: [I]1..=100 = read()
fn longer(x: Int, y: Int) -> Int: if y > x: y else: x
var l = 1
var g = 0
var best = 0
var each: [I]Int = 1
for i in I:
    l lcm= a[i]
    g gcd= a[i]
    best longer= a[i]
    best min= 1000
    each[i] lcm= a[i] * 2
print(l, g, best, each)
