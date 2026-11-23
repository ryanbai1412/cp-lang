// NOI 2017 P1, a huge integer under x += a * 2^b, with single-bit queries (buildable today-form)
// original: corpus/submissions/5690633.cpp
let q: 1..=1e6 = read()
let t1: Int = read()
let t2: Int = read()
let t3: Int = read()

// x = up - down, both only ever growing, in 30-bit digits; a carry chain on a
// number that only grows is amortised O(1)
const BASE = 1073741824  // 2^30
let digits = q + 4       // b <= 30 q, plus room for the carries
domain Digit = 0..digits
var up: [Digit]Int = 0
var down: [Digit]Int = 0

// highest digit where up and down differ
var top = SegTree<Digit, Int>(max, fill: -1)

fn mark(d: Digit):
    top.set(d, if up[d] != down[d]: d.ord else: -1)

for _ in 0..q:
    let kind: 1..=2 = read()
    if kind == 1:
        let a: Int = read()
        let b: 0..=30 * q = read()
        var d = b / 30
        var carry = abs(a) << (b % 30)
        while carry > 0:
            let dd = Digit(d)
            let total = (if a > 0: up[dd] else: down[dd]) + carry
            if a > 0: up[dd] = total % BASE
            else: down[dd] = total % BASE
            mark(dd)
            carry = total / BASE
            d += 1
    else:
        let k: 0..=30 * q + 60 = read()
        let (d, low) = (k / 30, (1 << (k % 30)) - 1)
        let dd = Digit(d)
        // subtracting bit by bit, bit k sees a borrow iff the low k bits of
        // `up` are less than those of `down`
        let (u, w) = (up[dd] & low, down[dd] & low)
        var borrow = u < w
        if u == w and d > 0:
            let h = top.fold(Digit(0)..dd)
            borrow = h >= 0 and up[Digit(h)] < down[Digit(h)]
        print(((up[dd] ^ down[dd]) >> (k % 30) & 1) ^ Int(borrow))
