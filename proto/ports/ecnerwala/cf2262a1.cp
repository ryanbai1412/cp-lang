// Codeforces 2262 A1: a[i] = floor(x / i) confines x to [a[i]*i, (a[i]+1)*i).
// Mark each bucket with a difference array and list the uncovered positions.
@!multitest
let n: 1..=2e5 = read()
let a: [1..=n]Int = read()
var diff: [0..=n]Int = 0
for i in 1..=n:
    diff[min(n, a[i] * i)] += 1
    diff[min(n, a[i] * i + i)] -= 1
let cover = scan(diff, +)
let free = [i for i in 0..n if cover[i] == 0]
print(free.len)
print(free)
