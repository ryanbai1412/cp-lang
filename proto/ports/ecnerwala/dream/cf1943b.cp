// Every length-1 substring is a palindrome. For any substring t = s[l..=r] of length m:
// - If t is constant, no length is good (0).
// - If t has period 2 (alternating), exactly the even lengths 2, 4, ..., 2*(m/2) are good.
// - Otherwise every length in 2..m is good, and length m is good iff t is not a palindrome.
@!multitest
use std.strings.manacher

let (n, q): (2..=200_000, 1..=200_000) = read()
domain Pos = 1..=n
let s: [Pos]Char = read()

let pal = manacher(s)
var alt_run: [Pos]Int = 1
for i in Pos where let prev2 = i.prev?.prev:
    if s[i] == s[prev2]: alt_run[i] = alt_run[prev2] + 1

fn non_pal_score(l: Pos, r: Pos) -> Int:
    let m = (l..=r).len
    let prev = r.prev!
    let has_period2 = alt_run[r] >= (m + 1) / 2 and alt_run[prev] >= m / 2
    if has_period2:
        if s[r] == s[prev]: return 0
        let evens = m / 2
        return evens * (evens + 1)
    let total = m * (m + 1) / 2 - 1
    if pal.is_palindrome(l..=r): total - m else: total

repeat q:
    let (l, r): (Pos, Pos) = read()
    print(non_pal_score(l, r))
