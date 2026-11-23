// CF 1930D1 Sum over all Substrings (easy): f greedily puts a 1 right after each uncovered
// '1', covering three pattern positions; f of every prefix of s[l..] falls out of one scan.
@!multitest
let n: 1..=100 = read()
let s: Str = read()
var total = 0
for l in 0..n:
    var ones = 0
    var covered_to = l
    for j in l..n:
        if j >= covered_to and s[j] == '1':
            ones += 1
            covered_to = j + 3
        total += ones
print(total)
