// Every length-1 substring is a palindrome. For any substring t = s[l..=r] of length m:
// - If t is constant, no length is good (0).
// - If t has period 2 (alternating), exactly the even lengths 2, 4, ..., 2*(m/2) are good.
// - Otherwise every length in 2..m is good, and length m is good iff t is not a palindrome.
@!multitest
use std.strings.manacher

let n: 2..=200_000 = read()
let q: 1..=200_000 = read()
domain Pos = 1..=n
let s_str: Str = read()
let s: [Pos]Char = [Pos] i => s_str[i.ord]

let pal = manacher(s)
var alt_run: [Pos]Int = 1
for i in Pos:
    if let p1 = i.prev:
        if let p2 = p1.prev:
            if s[i] == s[p2]: alt_run[i] = alt_run[p2] + 1

fn non_pal_score(l: Pos, r: Pos) -> Int:
    let m = r.ord - l.ord + 1
    let prev = r.prev!
    let has_period2 = alt_run[r] >= (m + 1) / 2 and alt_run[prev] >= m / 2
    if has_period2:
        if s[r] == s[prev]: return 0
        let evens = m / 2
        return evens * (evens + 1)
    let total = m * (m + 1) / 2 - 1
    if pal.is_palindrome(l.ord, r.ord + 1): total - m else: total

repeat q:
    let (l, r): (Pos, Pos) = read()
    print(non_pal_score(l, r))
