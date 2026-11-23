// Source: Codeforces 2150E1, Hidden Single (Easy), https://codeforces.com/contest/2150/problem/E1
// Problem: a hidden array of length 2n - 1 (n <= 300) holds each of 1..=n twice except one
//   value once. Ask whether x occurs at some position of a set S; find the single value in at
//   most 4n + 2 ceil(log2 n) queries. Interactive multitest, sum n^2 <= 4e5.
// Solution: shuffle positions with a fixed seed, then binary-search a range [lo, hi) holding
//   the single, knowing each value's copies in it (None, One, Both; the single counts as Both).
//   Per halving, ask each Both value on the left half and, if present, on the right; ask each
//   One value on the right. The single lies in the half whose counted copies exceed its
//   length; stop when one Both value is left. About 4n queries, O(n^2) time.

@!interactive
use std.rand.Rng

enum Copies: None | One | Both

fn weight(c: Copies) -> Int = match c:
    Copies.None => 0
    Copies.One => 1
    Copies.Both => 2

fn complement(total: Copies, left: Copies) -> Copies = match weight(total) - weight(left):
    0 => Copies.None
    1 => Copies.One
    _ => Copies.Both

const SHUFFLE_SEED = 48
var rng = Rng.new(SHUFFLE_SEED)

@!multitest
let n: 1..=300 = read()
domain Val = 1..=n
domain Pos = 0..2 * n - 1

var perm: [Pos]Int = [Pos] p => p.ord + 1
rng.shuffle(mut perm)

fn has(v: Val, lo: Pos.bounds, hi: Pos.bounds) -> Bool:
    if lo == hi: return false
    print("?", v, hi.ord - lo.ord, perm[lo..hi])
    read<Int>() == 1

var copies: [Val]Copies = Copies.Both
var (lo, hi) = (Pos.bounds.first!, Pos.bounds.last!)
var alive = Val.collect()

while alive.len > 1:
    let mid = Pos.cuts.nth((lo.ord + hi.ord) / 2)
    var left: [Val]Copies = Copies.None
    for v in alive:
        left[v] = if not has(v, lo, mid):
            Copies.None
        elif has(v, mid, hi):
            Copies.One
        else:
            Copies.Both
    for v in Val if copies[v] == Copies.One:
        left[v] = if has(v, mid, hi): Copies.None else: Copies.One

    let left_weight = [weight(left[v]) for v in Val].sum()
    if left_weight > mid.ord - lo.ord:
        hi = mid
        copies = left
    else:
        lo = mid
        copies = [Val] v => complement(copies[v], left[v])
    alive = [v for v in Val if copies[v] == Copies.Both]

print("!", alive[0])
