// Codeforces 1901C Add, Divide and Floor
// in: t; per test n, then a (n ints); out: per test, the minimum number of operations, then the chosen x values on the next line when there are at most n
@!multitest
let n: 1..=2e5 = read()
let a = read<[n]0..=1e9>()

var lo = a.min()!
var hi = a.max()!
var xs = List<Int>()
while lo != hi:
    let x = lo % 2
    xs.push(x)
    lo = (lo + x) / 2
    hi = (hi + x) / 2
print(xs.len)
if xs.len <= n: print(xs)
