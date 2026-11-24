// n 20-letter words read into a list that one comprehension walks for the last time,
// then a 4n-cell table: the words are freed before the table is built.
let n: 1..=400_000 = read()
let words = read<[n]Str>()
let keys = [w.len * 31 for w in words]
var big: List<Int> = List.filled(4 * n, 0)
for i in 0..4 * n:
    big[i] = keys[i % n] + i
print(big[4 * n - 1] + big[0])
