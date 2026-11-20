// Codeforces 2268 B: count the values with an even number of set bits,
// before any update and after each point assignment.
@!multitest
let (n, q): (2..=2e5, 0..=2e5) = read()
var a: [1..=n]Int = read()
fn even(x: Int) -> Int: Int(popcount(x) % 2 == 0)
var good = sum(even(x) for x in a)
var out = [good]
for _ in 0..q:
    let (p, x): (1..=n, Int) = read()
    good += even(x) - even(a[p])
    a[p] = x
    out.push(good)
print(out)
