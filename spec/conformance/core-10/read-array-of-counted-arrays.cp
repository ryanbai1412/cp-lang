// expect: ok, output "6 1"
// rule: 10.2.1
// input: "1 2 3\n4 5 6\n"
// `[k][n]T` reads k rows of n tokens, row-major (rebuild round 19 H01).
let x = read<[2][3]Int>()
print(x[1][2], x[0][0])
