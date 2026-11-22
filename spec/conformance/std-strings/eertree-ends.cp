// expect: ok, output "YES YES YES\nYES\nYES"
use std.strings.{PalindromicTree}
domain L = 'a'..='c'
fn w(s: Str) -> List<L> = [L(c) for c in s]
var t = PalindromicTree<L>.of(w("ba"))
var u = PalindromicTree<L>.of(w("ab"))
u.pop()
var v = PalindromicTree<L>()
v.push(L('a'))
print(yes(t.push_front(L('a')) == 4), yes(u.distinct == 1 and u.last_state == 2), yes(v.push(L('a')) == 3))
fn brute(s: List<L>) -> (Int, Int):
    var seen = List<List<L>>()
    var occ = 0
    for i in 0..s.len:
        for j in i + 1..=s.len:
            let w = [s[k] for k in i..j]
            if w == [w[w.len - 1 - k] for k in 0..w.len]:
                occ += 1
                if not seen.contains(w): seen.push(w)
    (seen.len, occ)
var ok = true
var seed = 12345
for round in 0..60:
    var p = PalindromicTree<L>()
    var s = List<L>()
    for step in 0..12:
        seed = (seed * 1103515245 + 12345) % 2147483648
        let ch = L(if (seed / 65536) % 3 == 0: 'a' else: if (seed / 65536) % 3 == 1: 'b' else: 'c')
        if round % 2 == 0 and (seed / 8) % 2 == 0:
            p.push_front(ch)
            s = [ch] + s
        else:
            p.push(ch)
            s.push(ch)
        let (d, o) = brute(s)
        let counts = p.occ_counts()
        var total = 0
        for k in 2..counts.len: total += counts[k]
        if p.distinct != d or total != o: ok = false
print(yes(ok))
var q = PalindromicTree<L>()
var r = List<L>()
var good = true
for step in 0..200:
    seed = (seed * 1103515245 + 12345) % 2147483648
    let ch = L(if (seed / 65536) % 2 == 0: 'a' else: 'b')
    if r.len > 0 and (seed / 8) % 3 == 0:
        q.pop()
        r.pop()
    else:
        q.push(ch)
        r.push(ch)
    let (d, _) = brute(r)
    if q.distinct != d: good = false
print(yes(good))
