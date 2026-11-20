// NOI 2017 P1, a huge integer under x += a * 2^b, with single-bit queries (buildable today-form)
// original: corpus/submissions/5690633.cpp
// "Last digit below d where the two numbers differ" is a hand-rolled max
// tree: the prototype's Set has no last_lt.
let q: 1..=1e6 = read()
let t1: Int = read()
let t2: Int = read()
let t3: Int = read()

// x = up - down, both only ever growing, in 30-bit digits; a carry chain on a
// number that only grows is amortised O(1)
const BASE = 1073741824             // 2^30
let digits = q + 4                      // b <= 30 q, plus room for the carries
var up: [0..digits]Int = 0
var down: [0..digits]Int = 0

// top[k]: the highest digit under tree node k where up and down differ
var size = 1
while size < digits: size *= 2
var top: [0..2 * size]Int = -1

fn mark(d: Int):
    var k = d + size
    top[k] = if up[d] != down[d]: d else: -1
    while k > 1:
        k /= 2
        top[k] = max(top[2 * k], top[2 * k + 1])

fn highest_below(d: Int) -> Int:       // over digits 0..d
    var (l, r, best) = (size, size + d, -1)
    while l < r:
        if l % 2 == 1:
            best max= top[l]
            l += 1
        if r % 2 == 1:
            r -= 1
            best max= top[r]
        l /= 2
        r /= 2
    best

for _ in 0..q:
    let kind: 1..=2 = read()
    if kind == 1:
        let a: Int = read()
        let b: 0..=30 * q = read()
        var d = b / 30
        var carry = abs(a) << (b % 30)
        while carry > 0:
            let total = (if a > 0: up[d] else: down[d]) + carry
            if a > 0: up[d] = total % BASE
            else: down[d] = total % BASE
            mark(d)
            carry = total / BASE
            d += 1
    else:
        let k: 0..=30 * q + 60 = read()
        let (d, low) = (k / 30, (1 << (k % 30)) - 1)
        // subtracting bit by bit, bit k sees a borrow iff the low k bits of
        // `up` are less than those of `down`
        let (u, w) = (up[d] & low, down[d] & low)
        var borrow = u < w
        if u == w:
            let h = highest_below(d)
            borrow = h >= 0 and up[h] < down[h]
        print(((up[d] ^ down[d]) >> (k % 30) & 1) ^ Int(borrow))
