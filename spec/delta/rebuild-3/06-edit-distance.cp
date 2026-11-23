// CSES 1639 Edit Distance: fewest inserts, deletes and replaces turning one string into another.
// In: two lines, strings a and b. Out: the edit distance.
let a = read<Str>()
let b = read<Str>()
var d: [0..=a.len, 0..=b.len]Int = 0
for i in 0..=a.len, j in 0..=b.len:
    if i == 0 or j == 0: d[i, j] = i + j
    else:
        let replace = d[i - 1, j - 1] + (if a[i - 1] == b[j - 1]: 0 else: 1)
        d[i, j] = min(min(d[i - 1, j], d[i, j - 1]) + 1, replace)
print(d[a.len, b.len])
