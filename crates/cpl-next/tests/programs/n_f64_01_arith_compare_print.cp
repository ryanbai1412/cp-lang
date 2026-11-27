// expect: ok, output "0.3000000000 YES NO 1.0000000000 -2.5000000000 7.0000000000 0.3333333333 3.0000000000 YES 1000.0000000000"
// rule: R01.4.2, R02.1, R10.3.4
// `F64` is IEEE in both engines: `0.1 + 0.2` is not `0.3`, and prints with ten decimals.
let a = 0.1 + 0.2
let half = 0.5
let n = 3
let k: F64 = 1e3
print(a, yes(a > 0.3), yes(a == 0.3), half * 2, -half - 2, half + 6.5, 1 / F64(n), F64(n), yes(half < 15e-1), k)
