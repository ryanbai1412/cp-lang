// Codeforces 2262 A1: a[i] = floor(x / i) confines x to the half-open bucket
// [a[i] * i, (a[i] + 1) * i).  Mark every bucket with a difference array;
// the positions in 0..n that no bucket covers are printed.
@!multitest
let n: 1..=2e5 = read()
let a: [1..=n]Int = read()   // a[i] belongs to divisor i
var cover: [0..=n]Int = 0
for i in 1..=n:
    cover[min(n, a[i] * i)] += 1
    cover[min(n, a[i] * i + i)] -= 1
for i in 0..n: cover[i + 1] += cover[i]
let free = [i for i in 0..n if cover[i] == 0]
print(free.len)
print(free)
