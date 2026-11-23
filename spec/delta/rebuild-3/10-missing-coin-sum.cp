// CSES 2183 Missing Coin Sum: smallest sum that no subset of the coins makes.
// In: n, then x1..xn. Out: the smallest unmakeable sum.
let n = read<1..=2e5>()
var reach = 0                       // every sum in 1..=reach can be made
for x in read<[n]1..=1e9>().sorted():
    if x > reach + 1: break
    reach += x
print(reach + 1)
