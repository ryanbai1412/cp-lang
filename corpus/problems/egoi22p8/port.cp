// EGOI '22 P8 (Rotation-Proof Cards, CP Lang 0.2.2): encode a number as segments
// on a 3x3 dot grid surviving 90-degree rotations, and decode back.
// original: corpus/submissions/5202737.cpp

fn turn(p: Int) -> Int = (2 - p % 3) * 3 + p / 3
fn seg_key(p: Int, q: Int) -> Int = min(p, q) * 9 + max(p, q)

const QUADS = 4
const SEGS = 7

fn ext1(a: Int) -> Int:
    let f = 127 - a
    f * f * f + f * f + f + 1 + f * (f + 1) / 2
fn ext2(a: Int, b: Int) -> Int:
    let f = 127 - a
    if b < a: return 0
    if b == a: return f * f + f + 1
    f * f + 128 - b
fn ext3(a: Int, b: Int, c: Int) -> Int:
    let f = 127 - a
    if b < a or c < a: return 0
    if c == a and b != a: return 128 - b
    if c == a: return f + 1
    f
fn ext4(a: Int, b: Int, c: Int, d: Int) -> Int:
    if b < a or c < a or d < a: return 0
    if a == c and b > a and d >= b: return 1
    if (a != c or a == b) and (a != d or a == c): return 1
    0

fn rank(a: Int, b: Int, c: Int, d: Int) -> Int:
    var r = 0
    for i in 0..a: r += ext1(i)
    for i in 0..b: r += ext2(a, i)
    for i in 0..c: r += ext3(a, b, i)
    for i in 0..=d: r += ext4(a, b, c, i)
    r

fn rank_packed(x: Int) -> Int =
    rank(x / 128 / 128 / 128, x / 128 / 128 % 128, x / 128 % 128, x % 128)

let ends: List<Int> = [6, 7, 7, 8, 4, 7, 4, 6, 7, 5, 0, 7, 0, 5]
var seg: [0..QUADS, 0..SEGS](Int, Int) = (0, 0)
for j in 0..SEGS:
    seg[0, j] = (ends[2 * j], ends[2 * j + 1])
    for i in 1..QUADS: seg[i, j] = (turn(seg[i - 1, j].0), turn(seg[i - 1, j].1))
for i in 0..QUADS:
    for j in 0..SEGS: seg[i, j] = (min(seg[i, j].0, seg[i, j].1), max(seg[i, j].0, seg[i, j].1))

let tests: Int = read()
for _ in 0..tests:
    let kind: 1..=2 = read()
    if kind == 1:
        let n: 1..=67_000_000 = read()
        let card = (0..(1 << 28)).first_true(x => rank_packed(x) >= n)!
        var out: List<(Int, Int)> = []
        for i in 0..QUADS:
            let mask = (card >> (7 * (3 - i))) & 127
            for j in 0..SEGS:
                if ((mask >> j) & 1) == 1: out.push(seg[i, j])
        print(out.len)
        for (p, q) in out: print(p / 3, p % 3, q / 3, q % 3)
    else:
        let k: 0..=28 = read()
        var drawn: [0..81]Bool = false
        for _ in 0..k:
            let (x1, y1, x2, y2): (Int, Int, Int, Int) = read()
            drawn[seg_key(3 * x1 + y1, 3 * x2 + y2)] = true
        var v = 0
        for i in 0..QUADS:
            for j in 0..SEGS:
                if drawn[seg_key(seg[i, j].0, seg[i, j].1)]: v |= 1 << (7 * (3 - i) + j)
        var best: ?Int = none
        for _ in 0..QUADS:
            v = ((v << 7) & ((1 << 28) - 1)) | (v >> 21)
            best min= v
        print(rank_packed(best!))
