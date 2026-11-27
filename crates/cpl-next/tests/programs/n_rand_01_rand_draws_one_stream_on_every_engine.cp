// expect: ok, output "1 -1 5 -7261621834428681060\n539"
// rule: R02.10.1
// `rand(lo, hi)` is an ordinary core fn over one program-wide splitmix64 stream, so every engine draws the same numbers.
let a = rand(1, 6)
let b = rand(-3, 3)
let c = rand(5, 5)
let d = rand(-9223372036854775807, 9223372036854775807)
print(a, b, c, d)
var s = 0
for i in 0..1000: s += rand(0, 1)
print(s)
