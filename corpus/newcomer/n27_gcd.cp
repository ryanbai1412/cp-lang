let n: 1..=100 = read()
let a: [0..n]1..=1000000000 = read()
fn g(x: Int, y: Int) -> Int:  // expect: hand_rolled_gcd
    if y == 0:
        return x
    return g(y, x % y)
var r = 0
for x in a:  // expect: loop_reduction
    r = g(r, x)
print(r)
