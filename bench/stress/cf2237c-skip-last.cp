// Codeforces 2237 C (proto/ports/ecnerwala/cf2237c.cp), with a planted
// off-by-one at the end of the array: the loop stops one value early.
fn total(a: List<Int>) -> Int:
    var cur = a[0]
    for x in a[1..a.len - 1]: cur = if x >= cur: x else: cur + x
    cur

@alt(total)
fn total_slow(a: List<Int>) -> Int:
    var cur = a[0]
    for x in a[1..a.len]: cur = if x >= cur: x else: cur + x
    cur

@!multitest
let n: 1..=2e5 = read()
let a: [0..n]1..=1e9 = read()
print(total([x for x in a]))
