// expect: ok, output "9"
// input: "3\n5 9 2\n"
let n: 1..=100 = read()
let a: [0..n]Int = read()
var best = -1000000000000000000  // expect: sentinel_scalar
for x in a:
    if x > best:  // expect: chmin_chmax_if
        best = x
print(best)
