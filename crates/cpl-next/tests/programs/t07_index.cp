// expect: R0101
// input: "5\n"
let n: Int = read()
var xs: List<Int> = [1, 2, 3]
print(xs[n - 4] + xs[n])  // <- here
