// Codeforces 2183 I2: Pairs Flipping (Hard Version).
// Split at an odd mid: the left part pairs cells outward from its centre with the even lengths,
// the right part pairs its ends inward with the odd ones. Equal pairs clear; an unequal pair hands
// its 1 on as the carry, which a later length cancels where it lands. Where it cannot, the
// editorial's frame replays (or a 20-cell endgame table) leave one 1 per loss: at most 7 (stress: 4).

const W = 20
const MASKS = 1 << W
enum Step: Skip, At(Int)

// endgame[k][s]: the next move that clears the W-cell window s down to at most one 1
// using each odd length 2k + 1, 2k + 3, ..., W - 1 once; tabulated backwards from the end.
fn tabulate_endgame() -> [0..=W / 2][0..MASKS]?Step:
    var t: [0..=W / 2][0..MASKS]?Step = none
    t[W / 2][0] = Skip
    for i in 0..W: t[W / 2][1 << i] = Skip
    for k in (0..W / 2).rev:
        let len = 2 * k + 1
        for s in 0..MASKS:
            if t[k + 1][s] == none: continue
            t[k][s] ??= Skip
            for i in 0..W - len: t[k][s ^ (1 << i) ^ (1 << (i + len))] ??= At(i)
    t
let endgame = tabulate_endgame()

fn run(lo: Int, hi: Int, by: Int, offsets: List<Int>) -> List<Int> =
    [i + o for i in (lo..=hi).step(by) for o in offsets]

// The editorial's replay orders for a frame with A = B - 1 or A = B + 1, as offsets from the
// carry's end, keyed by the gap's end cells nearest to and farthest from the carry.
fn replay(d: Int, near: Bool, far: Bool, b: Int) -> List<Int> = match (d, near, far):
    (-1, false, false) => run(0, b - 4, 1, [0]) + [2 * b - 5, b - 3]
    (-1, true, false) => [0] + run(3, b - 3, 2, [0, -1]) + [b - 1, 1]
    (-1, false, true) => run(0, b - 3, 1, [0]) + [2 * b - 3]
    (-1, true, true) => run(1, b - 2, 1, [0]) + [0]
    (1, false, false) => run(1, b, 1, [0]) + [0]
    (1, true, false) => run(1, b - 1, 1, [0]) + [0, b]
    (1, false, true) => run(1, b - 1, 2, [0, -1]) + [2 * b]
    (1, true, true) => match b % 3:
        2 => [0, 1] + run(3, b - 2, 3, [2, 0, 1]) + [2]
        0 => [1, 0] + run(3, b - 3, 3, [1, 2, 0]) + [b, 2]
        _ => [1, 0] + run(4, b - 3, 3, [0, 1, -1]) + [b, b - 1, 2]

@!multitest
let n: 3..=2_000_000 = read()
let s: Str = read()
domain Len = 1..=n / 2
var bit: [1..=n]Bool = [1..=n] i => s[i - 1] == '1'
var at: [Len]?Int = none

fn flip(i: Int, j: Int):
    let (lo, hi) = (min(i, j), max(i, j))
    at[hi - lo] = lo
    bit[lo] ^= true
    bit[hi] ^= true

fn free(len: Len):
    if let lo = at[len]:
        bit[lo] ^= true
        bit[lo + len] ^= true
        at[len] = none

fn lone(l: Int, r: Int) -> ?Int = if bit[l]: l elif bit[r]: r else: none

// The frame [x, y] mirrors the carry p across the gap [l, r]: free the lengths its outer pairs
// spent and spend them again, longest first, in the editorial's order.
fn reframe(l: Int, r: Int, p: Int, x: Int, y: Int):
    let cell = (o: Int) => if p < l: x + o else: y - o
    let (a, b) = (l - x, r - l + 1)
    for len in (r - l + 2..=y - x - 2).step(2): free(len)
    var len = y - x - 2
    for o in replay(a - b, bit[cell(a)], bit[cell(a + b - 1)], b):
        flip(cell(o), cell(o + len))
        len -= 2

fn finish(l: Int):
    var s = (0..W).sum(i => Int(bit[l + i]) << i)
    for k in 0..W / 2:
        if let At(i) = endgame[k][s]!:
            flip(l + i, l + i + 2 * k + 1)
            s ^= (1 << i) ^ (1 << (i + 2 * k + 1))

// An even n spends its longest length on its last cell, then plays as odd n - 1.
var last = n
if n % 2 == 0:
    if bit[n]: flip(n / 2, n)
    last -= 1
let mid = if (last + 1) / 2 % 2 == 1: (last + 1) / 2 else: (last + 1) / 2 - 1

let c = mid / 2 + 1
var carry = lone(c, c)
for d in 1..c:
    let (l, r) = (c - d, c + d)
    if bit[l] and bit[r]: flip(l, r)
    elif bit[l] != bit[r]:
        if let p = carry: flip(p, p + 2 * d)
        carry = lone(l, r)

var (l, r) = (mid + 1, last)
carry = none
var fresh = true
while l < r:
    let some(p) = carry else:
        if bit[l] and bit[r]: flip(l, r)
        carry = lone(l, r)
        (l, r, fresh) = (l + 1, r - 1, true)
        continue
    if r - l + 1 == W:
        finish(l)
        break
    let (x, y) = (min(p, l + r - p), max(p, l + r - p))
    if fresh and r - l + 1 >= 10 and abs((l - x) - (r - l + 1)) == 1:
        reframe(l, r, p, x, y)
        carry = (x..=l).chain(r..=y).find(i => bit[i])
        fresh = false
    elif bit[l] and bit[r]:
        flip(l, r)
    else:
        if let q = [p + (r - l), p - (r - l)].find(q => l < q < r): flip(p, q)
        carry = lone(l, r)
        fresh = true
    (l, r) = (l + 1, r - 1)

print(Len.map(len => at[len] ?? 0))
