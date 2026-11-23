// CSES 1621 Distinct Numbers
// in: n, then n integers; out: the number of distinct values
let n = read<Int>()
print(read<[n]Int>().distinct().len)
