// CSES 1145 Increasing Subsequence: the length of the longest strictly increasing subsequence.
let n: 1..=2e5 = read()
let a = read<[n]1..=1e9>()
var tails: List<Int> = []
for v in a:
    let k = Int(tails.keys.bsearch(i => tails[i] >= v))
    if k == tails.len: tails.push(v) else: tails[k] = v
print(tails.len)
