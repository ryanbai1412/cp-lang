// CSES 1145 Increasing Subsequence: the length of the longest strictly increasing subsequence.
let n: 1..=2e5 = read()
let a: [0..n]1..=1e9 = read()
var tails: List<Int> = []
for v in a:
    if let k = tails.first_ge(v): tails[k] = v else: tails.push(v)
print(tails.len)
