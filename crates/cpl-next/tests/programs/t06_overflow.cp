// expect: R0202
// input: "5\n"
let n: Int = read()
var x = 1
for i in 0..n:
    print(x)
    x = x * 1000000007  // <- here
