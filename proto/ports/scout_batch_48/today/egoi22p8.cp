// EGOI '22 P8 (rotation-proof cards): encode a number as a set of
// segments on a 3x3 dot grid so that it survives rotation of the card, and
// decode it back. A card is four 7-bit quadrant masks (each quadrant is the
// previous one turned 90 degrees); the number is the rank of the card's
// smallest rotation among all canonical cards.
// Runnable on proto/cpc today: `1 n` prints the pattern, `2 k` + k segments
// prints the number.
// original: corpus/submissions/5202737.cpp (BuildPattern / GetCardNumber)

// dots are numbered 3 * x + y; turning the card maps (x, y) to (2 - y, x)
fn turn(p: Int) -> Int: return (2 - p % 3) * 3 + p / 3
fn seg_key(p: Int, q: Int) -> Int: return min(p, q) * 9 + max(p, q)

const QUADS = 4
const SEGS = 7
let ends: List<Int> = [6, 7, 7, 8, 4, 7, 4, 6, 7, 5, 0, 7, 0, 5]
var seg: [0..QUADS, 0..SEGS](Int, Int) = (0, 0)
for j in 0..SEGS:
    seg[0, j] = (ends[2 * j], ends[2 * j + 1])
    for i in 1..QUADS: seg[i, j] = (turn(seg[i - 1, j].0), turn(seg[i - 1, j].1))
// a segment is unordered: keep the smaller dot first
for i in 0..QUADS:
    for j in 0..SEGS: seg[i, j] = (min(seg[i, j].0, seg[i, j].1), max(seg[i, j].0, seg[i, j].1))

// number of canonical cards (a, b, c, d) that extend the given prefix;
// canonical means no rotation is lexicographically smaller
fn ext1(a: Int) -> Int:
    let f = 127 - a
    return f * f * f + f * f + f + 1 + f * (f + 1) / 2
fn ext2(a: Int, b: Int) -> Int:
    let f = 127 - a
    if b < a: return 0
    if b == a: return f * f + f + 1
    return f * f + 128 - b
fn ext3(a: Int, b: Int, c: Int) -> Int:
    let f = 127 - a
    if b < a or c < a: return 0
    if c == a and b != a: return 128 - b
    if c == a: return f + 1
    return f
fn ext4(a: Int, b: Int, c: Int, d: Int) -> Int:
    if b < a or c < a or d < a: return 0
    if a == c and b > a and d >= b: return 1
    // equal-to-a quadrants must form a prefix of (b, c, d)
    if (a != c or a == b) and (a != d or a == c): return 1
    return 0

// rank (1-based) of the last canonical card <= (a, b, c, d)
fn rank(a: Int, b: Int, c: Int, d: Int) -> Int:
    var r = 0
    for i in 0..a: r += ext1(i)
    for i in 0..b: r += ext2(a, i)
    for i in 0..c: r += ext3(a, b, i)
    for i in 0..=d: r += ext4(a, b, c, i)
    return r

fn rank_packed(x: Int) -> Int:
    return rank(x / 128 / 128 / 128, x / 128 / 128 % 128, x / 128 % 128, x % 128)

domain Card = 0..(1 << 28)
let t: Int = read()
for _ in 0..t:
    let kind: 1..=2 = read()
    if kind == 1:
        let n: 1..=67_000_000 = read()
        let card = Card.first_where(x => rank_packed(x.idx) >= n)!
        var out: List<(Int, Int)> = []
        for i in 0..QUADS:
            let mask = card.idx >> (7 * (3 - i)) & 127
            for j in 0..SEGS:
                if mask >> j & 1 == 1: out.push(seg[i, j])
        print(out.len)
        for (p, q) in out: print(p / 3, p % 3, q / 3, q % 3)
    else:
        let k: 0..=28 = read()
        var drawn: [0..81]Bool = false
        for _ in 0..k:
            let x1: 0..=2 = read()
            let y1: 0..=2 = read()
            let x2: 0..=2 = read()
            let y2: 0..=2 = read()
            drawn[seg_key(3 * x1 + y1, 3 * x2 + y2)] = true
        var best = 1 << 28
        var v = 0
        for i in 0..QUADS:
            for j in 0..SEGS:
                if drawn[seg_key(seg[i, j].0, seg[i, j].1)]: v |= 1 << (7 * (3 - i) + j)
        for _ in 0..QUADS:
            v = (v << 7 & ((1 << 28) - 1)) | v >> 21
            best = min(best, v)
        print(rank_packed(best))
