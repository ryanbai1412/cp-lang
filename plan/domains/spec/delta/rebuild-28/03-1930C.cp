// CF 1930C Lexicographically Largest: every a_i + i is reachable; sorted descending, each
// value is pushed down to stay strictly below the previous one.
@!multitest
let n: 1..=3e5 = read()
let a = read<[n]1..=1e9>()
var b = [a[i] + i + 1 for i in 0..n]
b.sort(desc: true)
for i in 1..n:
    b[i] min= b[i - 1] - 1
print(b)
