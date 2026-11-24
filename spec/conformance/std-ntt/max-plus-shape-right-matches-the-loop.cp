// expect: ok, output "5 8 9 10 11 11 1"
// rule: std.ntt.convolve

let a = [5, 1, 7]
let b = [0, 3, 4, 4]
let c = convolve(a, b, over: Semiring(max, +), shape: Curved.Right)
print(c, Int(c == convolve(a, b, over: Semiring(max, +), shape: Curved.Neither)))
