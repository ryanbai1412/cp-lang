// Codeforces 2183 B (proto/ports/ecnerwala/cf2183b.cp), with a planted
// off-by-one at the start of the range: the search for the smallest missing
// value starts at 1, so a missing 0 is never found.
// class: off-by-one
fn missing(k: Int, a: List<Int>) -> Int:
    var has = List.filled(a.len + 1, false)
    for x in a if x <= a.len: has[x] = true
    first(x for x in 1..k - 1 if not has[x]) ?? (k - 1)

@brute(missing)
fn missing_slow(k: Int, a: List<Int>) -> Int =
    first(x for x in 0..k - 1 if not a.any(v => v == x)) ?? (k - 1)

@!multitest
let n: 1..=2e5 = read()
let k: 2..=n + 1 = read()
let a: [0..n]0..=n = read()
print(missing(k, [x for x in a]))
