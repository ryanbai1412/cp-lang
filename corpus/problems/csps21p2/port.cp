// Source: CSP-S 2021 P2 (Bracket Sequence), https://dmoj.ca/problem/csps21p2
// Problem: a pattern of n <= 500 characters from ( ) * ?, and k <= n. Count, mod 1e9+7, the
//   fillings of the ?s that are super bracket sequences: (), (S), AB, ASB, (AS), (SA) with
//   A, B super sequences and S a run of 1 to k stars (in () and (S), up to k stars inside).
// Solution: interval DP, each string split one way. wrap[l, r]: l and r match around only
//   stars, stars then a sequence, or a sequence then at least one star. seq[l, r] =
//   wrap[l, r] + sum over the end i of the first wrapped block of wrap[l, i] times
//   tail[i, r], 0 to k stars after i and then a sequence. A prefix count of the positions that
//   can't be stars answers "is this run all stars" in O(1). O(n^3).
// original: corpus/submissions/4819734.cpp
// Bounds: n: 1..=500; k: 1..=n
type Z = Mod<1_000_000_007>
let n: 1..=500 = read()
let k: 1..=n = read()
domain Pos = 0..n
let s: [Pos]Char = read()

fn can_be(i: Pos, c: Char) -> Bool = s[i] == '?' or s[i] == c
let not_star = Prefix([Pos] i => if can_be(i, '*'): 0 else: 1)
fn stars(run: Range<Pos>) -> Bool = run.len <= k and not_star.fold(run) == 0

var wrap: [Pos, Pos]Z = 0
var seq: [Pos, Pos]Z = 0
var tail: [Pos, Pos]Z = 0
for l in Pos.rev:
    for r in l.after:
        if can_be(l, '(') and can_be(r, ')'):
            let inside: Range<Pos> = l.r..r.l
            if stars(inside): wrap[l, r] += 1
            for i in inside:
                if stars(l.r..i.l): wrap[l, r] += seq[i, r.prev!]
                if i.r < r.l and stars(i.r..r.l): wrap[l, r] += seq[l.next!, i]
        for j in l..r:
            if stars(l.r..j.r): tail[l, r] += seq[j.next!, r]
        seq[l, r] = wrap[l, r] + sum(wrap[l, i] * tail[i, r] for i in l..r)
print(seq[Pos.first!, Pos.last!])
