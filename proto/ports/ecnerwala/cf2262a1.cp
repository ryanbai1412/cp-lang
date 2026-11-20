// Codeforces 2262 A1: a[i - 1] = floor(x / i) confines x to the half-open bucket
// [a[i - 1] * i, (a[i - 1] + 1) * i).  Mark every bucket with a difference array;
// the positions in 0..n that no bucket covers are printed.
@!multitest
let n: 1..=2e5 = read()
let a: [0..n]Int = read()   // a[i - 1] belongs to divisor i
var cover: [0..=n]Int = 0
for i in 1..=n:
    cover[min(n, a[i - 1] * i)] += 1
    cover[min(n, a[i - 1] * i + i)] -= 1
for i in 0..n: cover[i + 1] += cover[i]
let free = [i for i in 0..n if cover[i] == 0]
print(free.len)
print(free)
