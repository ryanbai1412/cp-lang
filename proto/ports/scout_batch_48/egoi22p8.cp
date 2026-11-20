// EGOI '22 P8 (rotation-proof cards): encode a number as a set of
// segments on a 3x3 dot grid so that it survives rotation of the card, and
// decode it back. A card is four 7-bit quadrant masks (each quadrant is the
// previous one turned 90 degrees); the number is the rank of the card's
// smallest rotation among all canonical cards.
// original: corpus/submissions/5202737.cpp (BuildPattern / GetCardNumber)

// dots are numbered 3 * x + y; turning the card maps (x, y) to (2 - y, x)
fn turn(p: Int) -> Int: return (2 - p % 3) * 3 + p / 3

const QUADS = 4
const SEGS = 7
// a segment is an unordered pair of dots, smaller first
type Seg = (Int, Int)
fn seg_of(p: Int, q: Int) -> Seg: return (min(p, q), max(p, q))
fn turned(s: Seg) -> Seg: return seg_of(turn(s.0), turn(s.1))
let base: [0..SEGS]Seg = [(6, 7), (7, 8), (4, 7), (4, 6), (5, 7), (0, 7), (0, 5)]
var seg: [0..QUADS, 0..SEGS]Seg = (0, 0)
for j in 0..SEGS:
    seg[0, j] = base[j]
    for i in 1..QUADS: seg[i, j] = turned(seg[i - 1, j])

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

const CARDS = 1 << 28

@grader
fn BuildPattern(n: 1..=67_000_000) -> List<(Pt, Pt)>:
    let card = bsearch_first(0..CARDS, x => rank_packed(x) >= n)!
    var out: List<(Pt, Pt)> = []
    for i in 0..QUADS:
        let mask = card >> (7 * (3 - i)) & 127
        for j in 0..SEGS:
            if mask >> j & 1 == 1:
                let (p, q) = seg[i, j]
                out.push((Pt(p / 3, p % 3), Pt(q / 3, q % 3)))
    return out

@grader
fn GetCardNumber(pattern: List<(Pt, Pt)>) -> Int:
    var drawn = Set<Seg>()
    for (p, q) in pattern: drawn.add(seg_of(3 * p.x + p.y, 3 * q.x + q.y))
    var v = 0
    for i in 0..QUADS:
        for j in 0..SEGS:
            if seg[i, j] in drawn: v |= 1 << (7 * (3 - i) + j)
    var best = v
    for _ in 1..QUADS:
        v = (v << 7 & (CARDS - 1)) | v >> 21
        best = min(best, v)
    return rank_packed(best)
