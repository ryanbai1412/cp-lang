// expect: ok, output "1180591620717411303424 -27 1 1"
// rule: R02.1; conformance core-08/pow-on-i128-past-int
// `**` on `I128`s runs past `Int`'s range, from the one op table on every runner.
print(I128(2) ** 70, I128(-3) ** 3, I128(-1) ** 5000000000, I128(0) ** 0)
