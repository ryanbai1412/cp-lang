// Codeforces 2150 E1 (interactive): find the unique value in 1..=n that appears
// once in a hidden array of length 2n - 1 (all other values appear twice).
//
// Shuffle the 2n - 1 positions with a fixed seed and binary-search [lo, hi):
// every value in 1..=n has a known count in [lo, hi) — None (0), One (1), or
// Both (2, assuming it is not the singleton). Splitting at mid:
// - Both: absent on left => (None, Both); else present on right => (One, One),
//   absent on right => (Both, None).
// - One: present on right => (None, One); absent on right => (One, None).
// Because every non-singleton contributes 2 units and the singleton contributes
// 1, the half whose assigned weight exceeds its length contains the singleton.

@!interactive
use std.rand.Rng

enum Copies: None | One | Both

fn weight(c: Copies) -> Int = match c:
    Copies.None => 0
    Copies.One => 1
    Copies.Both => 2

const SHUFFLE_SEED = 48
var rng = Rng.new(SHUFFLE_SEED)

@!multitest
let n: 1..=300 = read()
domain Val = 1..=n

var perm = List(1..2 * n)
rng.shuffle(mut perm)

fn has(v: Val, lo: Int, hi: Int) -> Bool:
    print("?", v, hi - lo, perm[lo..hi])
    read<Int>() == 1

var copies: [Val]Copies = Copies.Both
var (lo, hi) = (0, 2 * n - 1)
var alive = [v for v in Val]

while alive.len > 1:
    let mid = (lo + hi) / 2
    var left: [Val]Copies = Copies.None
    var right: [Val]Copies = Copies.None
    for v in alive:
        if not has(v, lo, mid):
            right[v] = Copies.Both
        elif has(v, mid, hi):
            left[v] = Copies.One
            right[v] = Copies.One
        else:
            left[v] = Copies.Both
    for v in Val if copies[v] == Copies.One:
        if has(v, mid, hi): right[v] = Copies.One else: left[v] = Copies.One

    let left_weight = [weight(left[v]) for v in Val].sum()
    if left_weight > mid - lo:
        hi = mid
        copies = left
    else:
        lo = mid
        copies = right
    alive = [v for v in Val if copies[v] == Copies.Both]

print("!", alive[0])
