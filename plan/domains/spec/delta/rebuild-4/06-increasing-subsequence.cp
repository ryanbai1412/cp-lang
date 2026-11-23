// CSES 1145 Increasing Subsequence
// in: n, then n integers; out: the length of the longest strictly increasing subsequence
let n = read<Int>()
print(read<[n]Int>().lis().len)
