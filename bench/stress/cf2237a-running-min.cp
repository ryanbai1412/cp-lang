// Codeforces 2237 A (proto/ports/ecnerwala/cf2237a.cp), with a planted
// multitest leak: the running minimum lives above the `@!multitest`
// divider, so a later test starts from an earlier test's minimum.
var lo = 1e9
fn prefix_min_sum(a: List<Int>) -> Int:
    var total = 0
    for x in a:
        lo min= x
        total += lo
    total

@brute(prefix_min_sum)
fn prefix_min_sum_slow(a: List<Int>) -> Int:
    var lo = a[0]
    var s = 0
    for x in a:
        lo min= x
        s += lo
    s

@!multitest
let n: 1..=2e5 = read()
let a: [0..n]1..=1e9 = read()
print(prefix_min_sum([x for x in a]))
