// expect: ok, output "0.333 -2.500\n0.3333333333 [  42] [-007] [0.50] [ -1.250] [00003.14]"
// rule: R10.3.2, R10.3.4
// `precision:` and the `{x:W}`, `{x:0W}` and `{x:.D}` specs lay numbers out alike in every engine.
let x = 1.0 / 3.0
print(x, -2.5, precision: 3)
let n = 42
let m = -7
let y = -1.25
print(x, "[{n:4}] [{m:04}] [{0.5:.2}] [{y:7.3}] [{3.14159:08.2}]")
