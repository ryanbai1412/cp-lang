// Source: Codeforces 2255E1 (What Will Remain at the End? (Easy Version)),
//   https://codeforces.com/contest/2255/problem/E1
// Problem: n, q <= 5e5 and an array a with values in {-1, 0, 1}. Operations, each making a
//   new version: 1 l r x (assign x in {-1, 0, 1}), 2 l r (negate), 3 l r (a = max(a, 0)),
//   4 p (print the max non-empty subarray sum of position p's values over all earlier
//   versions). Positions are XOR-encoded with the last answer mod 2^64. Multitest.
// Solution: each version appends every position's value to its history, kept as a MaxSub
//   summary. Negate and clamp see only the sign and scale with |value|, so a range tag maps
//   each sign to a new sign plus a history per unit of |value|; after an assignment the value
//   and the added history are constants. Tags compose in O(1): a SegTree with range apply and
//   point get, plus a `record` tag on the whole array after each operation, O((n + q) log n).
//   The code is the hard version's (any |a|, |x| <= 1e9).
// Tuned: the reference's dual tree by hand. Each node holds one flat `Tag` struct, the leaves
//   too (a leaf is always `set`), and `compose` updates the older tag in place, so no tag is
//   copied or built per push and every call is direct.
use std.algebra.{MaxSub}

struct Tag:
    var set: Bool
    var val: -1e9..=1e9
    var n0: Sign
    var n1: Sign
    var n2: Sign
    var p0: MaxSub
    var p1: MaxSub
    var p2: MaxSub
    var tail: MaxSub

type Sign = -1..=1

fn sign(v: Int) -> Sign = if v < 0: -1 elif v == 0: 0 else: 1
fn ns(t: Tag, z: Sign) -> Sign = if z < 0: t.n0 elif z == 0: t.n1 else: t.n2
fn pv(t: Tag, z: Sign) -> MaxSub = if z < 0: t.p0 elif z == 0: t.p1 else: t.p2

fn plain() -> Tag = Tag(
    false,
    0,
    -1,
    0,
    1,
    MaxSub.empty(),
    MaxSub.empty(),
    MaxSub.empty(),
    MaxSub.empty(),
)

fn signs(a: Sign, b: Sign, c: Sign) -> Tag:
    var t = plain()
    (t.n0, t.n1, t.n2) = (a, b, c)
    t

fn record() -> Tag:
    var t = plain()
    (t.p0, t.p1, t.p2) = (MaxSub.of(-1), MaxSub.of(0), MaxSub.of(1))
    t

fn assign(x: Int) -> Tag:
    var t = plain()
    (t.set, t.val) = (true, x)
    t

/// `a` then `b`, into `a`.
fn compose(mut a: Tag, b: Tag):
    if a.set:
        let z = sign(a.val)
        a.tail = a.tail + pv(b, z).scale(abs(a.val))
        a.val = abs(a.val) * ns(b, z)
    else:
        let (z0, z1, z2) = (a.n0, a.n1, a.n2)
        (a.n0, a.n1, a.n2) = (ns(b, z0), ns(b, z1), ns(b, z2))
        a.p0 = a.p0 + pv(b, z0)
        a.p1 = a.p1 + pv(b, z1)
        a.p2 = a.p2 + pv(b, z2)
    if b.set:
        if not a.set:
            a.tail = MaxSub.empty()
            a.set = true
        a.val = b.val
        a.tail += b.tail

@!multitest
let (n, q): (1..=500_000, 1..=500_000) = read()
let a: [0..n]-1e9..=1e9 = read()
var size = 1
var log = 0
while size < n:
    size *= 2
    log += 1
var seg = List.filled(2 * size, plain())
for i in 0..n:
    seg[size + i] = assign(a[i])
    seg[size + i].tail = MaxSub.of(a[i])
let blank = plain()
fn down(p: Int):
    unchecked:
        let t = seg[p]
        seg[p] = blank
        compose(mut seg[2 * p], t)
        compose(mut seg[2 * p + 1], t)
let rec = record()
let neg = signs(1, 0, -1)
let clamp = signs(0, 0, 1)
var last: U64 = 0
fn at(y: U64) -> Int = Int((y ^ last) % U64(n))
repeat q:
    let kind: 1..=4 = read()
    if kind == 4:
        let p = at(read()) + size
        for i in (1..=log).rev: down(p >> i)
        let ans = seg[p].tail.best
        print(ans)
        last = U64(ans)
    else:
        let (x, y) = (at(read()), at(read()))
        let f = if kind == 1: assign(read()) elif kind == 2: neg else: clamp
        var l = min(x, y) + size
        var r = max(x, y) + size + 1
        for i in (1..=log).rev:
            if (l >> i) << i != l: down(l >> i)
            if (r >> i) << i != r: down((r - 1) >> i)
        unchecked:
            while l < r:
                if l & 1 == 1:
                    compose(mut seg[l], f)
                    l += 1
                if r & 1 == 1:
                    r -= 1
                    compose(mut seg[r], f)
                l >>= 1
                r >>= 1
    compose(mut seg[1], rec)
