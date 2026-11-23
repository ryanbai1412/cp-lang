// CF 1881G Anya and the Mysterious String
// A string has no palindrome of length >= 2 iff no s[i] == s[i+1] and no
// s[i] == s[i+2]. Shifts are a range-add DualFenwick; a range add only
// changes those equalities at the six pairs that straddle its ends, which
// are kept as counts in two Fenwicks.
use std.fenwick.{DualFenwick}

@!multitest
let (n, m): (1..=2e5, 1..=2e5) = read()
domain P = 1..=n
let s = read<Str>()
var shift = DualFenwick([P] i => Int(s[Int(i) - 1]) - Int('a'))
var bad1 = Fenwick<P, Int>()
var bad2 = Fenwick<P, Int>()
fn ch(i: P) -> Int = shift.get(i) % 26
fn refresh(mut f: Fenwick<P, Int>, i: Int, gap: Int):
    if let a = P.at(i):
        if let b = P.at(i + gap): f.set(a, if ch(a) == ch(b): 1 else: 0)
for i in P:
    refresh(mut bad1, Int(i), 1)
    refresh(mut bad2, Int(i), 2)
for _ in 0..m:
    let tp = read<1..=2>()
    let (l, r) = read<(P, P)>()
    if tp == 1:
        shift.add(l..=r, read<1..=1e9>() % 26)
        for i in [Int(l) - 1, Int(r)]: refresh(mut bad1, i, 1)
        for i in [Int(l) - 2, Int(l) - 1, Int(r) - 1, Int(r)]: refresh(mut bad2, i, 2)
    else:
        let one = if l < r: bad1.fold(l..r) else: 0
        let two = if Int(r) - Int(l) >= 2: bad2.fold(l..P(Int(r) - 1)) else: 0
        print(yes(one + two == 0))
