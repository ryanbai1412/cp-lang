// expect: ok, output "30002 -7500"
// rule: 04.4.24
// input: "15000\n"
// A range whose ends are `+ - * /` of capped lets stores at the width its proven ends need, on both engines.
let n0: 1..=15_000 = read()
let n = n0 + 1
let m = n * (n + 1) / 2
var t: [0..m]0..=2 * n = 0
var h: [0..n]-(n / 2)..n - 1 = 0
t[m - 1] = 2 * n
h[0] = -(n / 2)
print(t[m - 1], h[0])
