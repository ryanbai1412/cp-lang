// NOI 2017 Queueing Worms: `n` worms, each with a length from 1 to 6, each a
// queue of one at first. "1 u v" puts the queue headed by `v` behind `u`, the
// last of its queue; "2 u" cuts the queue behind `u`; "3 s k" takes every
// block of `k` digits of `s`, counts the places where `k` worms standing in a
// row have those lengths, and prints the product of the counts modulo
// 998244353. `k` is at most 50 and there are few cuts.
//
// Maintain a count for every contiguous block of at most 50 worms. A join makes
// the blocks that cross `(u, v)` and a cut removes them: each crossing block is
// a suffix ending at `u` concatenated with a prefix starting at `v` of total
// length at most 50. With length-carrying `Hash` values and `+` concatenation,
// lengths travel with every hash and cannot be miscounted.

type Z = Mod<998_244_353>
let longest = 50
let one = [Hash.of(d) for d in 0..=6]

let (n, q): (1..=200_000, 1..=500_000) = read()
domain Worm = 1..=n
let size: [Worm]1..=6 = read()
var ahead: [Worm]?Worm = none
var behind: [Worm]?Worm = none
var blocks = Map<Hash, Int>()

fn count(block: Hash, by: Int):
    blocks[block] = (blocks.get(block) ?? 0) + by

/// Blocks of fewer than `longest` worms starting at `from` along `toward`
/// (`back = true`: extending on the right; `back = false`: prepending on the left).
fn reach(from: Worm, toward: [Worm]?Worm, back: Bool) -> List<Hash>:
    var out: List<Hash> = []
    var (at, block) = (some(from), Hash.empty)
    while out.len < longest - 1:
        if let w = at:
            block = if back: block + one[size[w]] else: one[size[w]] + block
            out.push(block)
            at = toward[w]
        else: break
    out

/// Count (`by = 1`) or forget (`by = -1`) every block with `v` right behind `u`.
fn joint(u: Worm, v: Worm, by: Int):
    let (left, right) = (reach(u, ahead, false), reach(v, behind, true))
    for a in left:
        for b in right:
            if a.len + b.len > longest: break
            count(a + b, by)

for w in Worm: count(one[size[w]], 1)
repeat q:
    match read<Int>():
        1 =>
            let (u, v): (Worm, Worm) = read()
            joint(u, v, 1)
            (behind[u], ahead[v]) = (some(v), some(u))
        2 =>
            let u: Worm = read()
            let v = behind[u]!
            joint(u, v, -1)
            (behind[u], ahead[v]) = (none, none)
        _ =>
            let (s, k): (Str, Int) = read()
            let h = RollingHash.new([c - '0' for c in s])
            var ways = Z(1)
            for i in 0..=s.len - k:
                ways *= Z(blocks.get(h.slice(i, i + k)) ?? 0)
            print(ways)
