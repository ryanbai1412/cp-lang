// How often each element's value occurs, with a planted index-space mix-up:
// the count table is indexed by value, and the fast fn reads it at the
// element's position instead.
// class: index-space
fn counts(a: List<Int>) -> List<Int>:
    var cnt = List.filled(a.len + 1, 0)
    for x in a: cnt[x] += 1
    [cnt[i + 1] for i in 0..a.len]

@brute(counts)
fn counts_slow(a: List<Int>) -> List<Int> =
    [a.count(y => y == x) for x in a]

let n: 1..=100 = read()
let a: [0..n]1..=n = read()
print(counts([x for x in a]))
