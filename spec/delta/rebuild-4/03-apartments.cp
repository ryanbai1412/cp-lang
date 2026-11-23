// CSES 1084 Apartments
// in: n m k, then n desired sizes, then m apartment sizes; out: the most applicants who get an apartment
let (n, m, k) = read<(Int, Int, Int)>()
var want = read<[n]Int>()
var size = read<[m]Int>()
want.sort()
size.sort()
var j = 0
var matched = 0
for w in want:
    while j < m and size[j] < w - k: j += 1
    if j < m and size[j] <= w + k:
        matched += 1
        j += 1
print(matched)
