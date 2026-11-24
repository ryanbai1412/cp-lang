use std.lazy.*

fn check(mut t: LazyTreap<Int, Int>, want: List<Int>) -> Bool:
    if t.len != want.len: return false
    for p in 0..want.len:
        if t.get(p) != want[p]: return false
    true

var ok = true
for n in 0..=40:
    var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f)
    var want: List<Int> = []
    for i in 0..n:
        t.insert_at(i / 2, i)
        want.insert(i / 2, i)
    for i in 0..n / 3:
        let at = (i * 7) % want.len
        let _ = t.erase_at(at)
        want.remove(at)
    ok = ok and check(mut t, want)
    let k = want.len / 2
    var back = t.split_at(k)
    ok = ok and check(mut t, want.take(k).collect()) and check(mut back, want.drop(k).collect())
    t.splice_back(mut back)
    ok = ok and check(mut t, want)
print(yes(ok))
