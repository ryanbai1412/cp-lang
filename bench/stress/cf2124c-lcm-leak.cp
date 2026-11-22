// Codeforces 2124 C (proto/ports/ecnerwala/cf2124c.cp), with a planted
// multitest leak: the answer starts above the `@!multitest` divider, so it
// keeps every factor an earlier test needed.
// class: multitest-leak
var res = 1
fn smallest(b: List<Int>) -> Int:
    for (x, y) in b.adjacent if y % x != 0:
        res = lcm(res, x / gcd(x, y))
    res

@brute(smallest)
fn smallest_slow(b: List<Int>) -> Int:
    var need = 1
    for (x, y) in b.adjacent if y % x != 0:
        need = lcm(need, x / gcd(x, y))
    need

@!multitest
let n: 2..=1e5 = read()
let b: [0..n]1..=1e9 = read()
print(smallest([x for x in b]))
