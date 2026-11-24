// Source: EGOI 2022 P8 (Rotation-Proof Cards), https://dmoj.ca/problem/egoi22p8
// limits: 2000 ms, 512 MB
// Problem: encode a number n <= 6.7e7 as a set of segments between dots of a 3x3 grid
//   so that the card decodes to the same number after any 90-degree rotation; a second
//   run decodes a drawn card back.
// Solution: 28 segments fall into 7 orbits of 4 under a quarter turn, so a card is 4 digits
//   of 7 bits (digit q holds the orbits' q-th turns) and a turn rotates the digits. Number
//   the cards that are the least of their four rotations: count those up to a card digit by
//   digit (`below`); encode binary searches the least card of rank n, decode ranks the least
//   rotation of the drawn card. O(28 * 128) per test.
// Stdin protocol (both runs in one file, as in spec.cp): t, then per test 1 and n (encode:
//   print the segment count and each segment as x1 y1 x2 y2) or 2, k and k segments (decode).
// original: corpus/submissions/5202737.cpp
// Bounds: kind: 1..=2; n: 1..=67_000_000; k: 0..=28; lines.a.r, lines.a.c, lines.b.r, lines.b.c: 0..=2

type Dot = (r: 0..3, c: 0..3)
type Seg = (a: Dot, b: Dot)  // a <= b

fn turn(d: Dot) -> Dot = (r: 2 - d.c, c: d.r)
fn seg(a: Dot, b: Dot) -> Seg = if a <= b: (a, b) else: (b, a)

const BITS = 28
const DIGIT = 128  // 7 bits

fn digit(card: Int, q: Int) -> Int = (card >> (7 * (3 - q))) % DIGIT

// Least-rotation cards whose leading digits are a, a b, a b c or a b c d (each with the
// later digits free); the four cases of the accepted C++.
fn with1(a: Int) -> Int:
    let f = 127 - a
    f * f * f + f * f + f + 1 + f * (f + 1) / 2
fn with2(a: Int, b: Int) -> Int:
    let f = 127 - a
    if b < a: 0 elif b == a: f * f + f + 1 else: f * f + 128 - b
fn with3(a: Int, b: Int, c: Int) -> Int:
    let f = 127 - a
    if b < a or c < a: 0 elif c == a and b != a: 128 - b elif c == a: f + 1 else: f
fn with4(a: Int, b: Int, c: Int, d: Int) -> Int:
    if b < a or c < a or d < a: 0
    elif a == c and a < b <= d: 1
    elif (a != c or a == b) and (a != d or a == c): 1
    else: 0

fn below(card: Int) -> Int:  // least-rotation cards <= card
    let (a, b, c, d) = (digit(card, 0), digit(card, 1), digit(card, 2), digit(card, 3))
    sum(with1(i) for i in 0..a) + sum(with2(a, i) for i in 0..b) +
        sum(with3(a, b, i) for i in 0..c) + sum(with4(a, b, c, i) for i in 0..=d)

fn turned(card: Int, t: Int) -> Int =
    ((card << (7 * t)) | (card >> (BITS - 7 * t))) % (1 << BITS)

let orbits: [0..7]Seg = [
    ((2, 0), (2, 1)), ((2, 1), (2, 2)), ((1, 1), (2, 1)), ((1, 1), (2, 0)),
    ((1, 2), (2, 1)), ((0, 0), (2, 1)), ((0, 0), (1, 2)),
]
var segs: [0..4, 0..7]Seg = orbits[0]
for j in 0..7:
    var s = orbits[j]
    for q in 0..4:
        segs[q, j] = s
        s = seg(turn(s.a), turn(s.b))

@!multitest
let kind: 1..=2 = read()
if kind == 1:
    let n: 1..=67_000_000 = read()
    let card = (0..(1 << BITS)).first_true(x => below(x) >= n)!
    let drawn = [segs[q, j] for q in 0..4 for j in 0..7 if (digit(card, q) >> j) % 2 == 1]
    print(drawn.len)
    for s in drawn: print(s.a.r, s.a.c, s.b.r, s.b.c)
else:
    let k: 0..=28 = read()
    let lines: [k](a: Dot, b: Dot) = read()
    let drawn = Set([seg(l.a, l.b) for l in lines])
    var card = 0
    for q in 0..4:
        for j in 0..7:
            if drawn.contains(segs[q, j]): card |= 1 << (7 * (3 - q) + j)
    print(below(min(turned(card, t) for t in 0..4)!))
