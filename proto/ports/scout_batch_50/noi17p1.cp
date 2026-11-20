// NOI 2017 P1, a huge integer under x += a * 2^b, with single-bit queries
// original: corpus/submissions/5690633.cpp
let (q, t1, t2, t3): (1..=1e6, Int, Int, Int) = read()

// x = up - down, both only ever growing, in 30-bit digits; a carry chain on a
// number that only grows is amortised O(1)
const BASE = 1 << 30
domain Digit = 0..q + 4                 // b <= 30 q, plus room for the carries
var num: [Bool, Digit]Int = 0           // num[true] is up, num[false] is down
var differ = Set<Digit>()               // digits where up and down disagree

for _ in 0..q:
    let kind: 1..=2 = read()
    if kind == 1:
        let (a, b): (Int, 0..=30 * q) = read()
        var (d, carry) = (Digit(b / 30), abs(a) << (b % 30))
        while carry > 0:
            let total = num[a > 0, d] + carry
            num[a > 0, d] = total % BASE
            if num[true, d] != num[false, d]: differ.add(d) else: differ.remove(d)
            (d, carry) = ((d + 1)!, total / BASE)
    else:
        let k: 0..=30 * q + 60 = read()
        let (d, r) = (Digit(k / 30), k % 30)
        // subtracting bit by bit, bit k sees a borrow iff the low k bits of
        // `up` are less than those of `down`: the highest differing place decides
        let (u, w) = (num[true, d] % (1 << r), num[false, d] % (1 << r))
        let borrow = if u != w: u < w
                     elif let h = differ.last_lt(d): num[true, h] < num[false, h]
                     else: false
        print((num[true, d] ^ num[false, d]) >> r & 1 ^ Int(borrow))
