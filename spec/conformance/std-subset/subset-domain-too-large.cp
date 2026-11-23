// expect: R0001 at 3
// input: "30\n"
// A `Subset` over more than 25 values traps where its domain is sized, in both engines.
let n: Int = read()
domain V = 0..n
var dp: [Subset<V>]Int = 0
print(len(dp))
