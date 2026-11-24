// Source: NOI 2017 P1, Integer, https://dmoj.ca/problem/noi17p1
// Problem: x starts at 0; n <= 1e6 operations: add a * 2^b to x (|a| <= 1e9, 0 <= b <= 30n;
//   x stays >= 0), or print bit k of x. The first line also has t1, t2, t3 (subtask hints).
// Solution: keep x = up - down with the positive and negative additions in separate base-2^30
//   digit arrays; each only grows, so carry chains are amortised O(1). Bit k of x is bit k of
//   up xor down xor a borrow from the lower bits, and the borrow is set iff up < down below
//   bit k: compare the low part of digit k / 30, and if equal, the highest lower digit where
//   up and down differ, from an or-SegTree of differing digits searched with left_while. O(n log n) amortised.
// original: corpus/submissions/5690633.cpp
let (q, _, _, _): (1..=1e6, Int, Int, Int) = read()
const BASE = 1 << 30
domain Digit = 0..q + 4  // b <= 30 q, plus room for the carries
var up: [Digit]0..BASE = 0
var down: [Digit]0..BASE = 0
var differ = SegTree<Digit, Bool>(or, fill: false)

fn add(x: Int, b: Int):
    var (d, carry) = (b / 30, abs(x) << (b % 30))
    while carry > 0:
        let total = (if x > 0: up[Digit(d)] else: down[Digit(d)]) + carry
        if x > 0: up[Digit(d)] = total % BASE
        else: down[Digit(d)] = total % BASE
        differ.set(Digit(d), up[Digit(d)] != down[Digit(d)])
        carry = total / BASE
        d += 1

/// Whether up - down borrows into bit k: the bits below k of up are less than those of down.
fn borrows(d: Digit, low: Int) -> Bool:
    let (u, w) = (up[d] & low, down[d] & low)
    if u != w: return u < w
    let c = differ.left_while(Digit.cuts(d.ord), x => not x)
    c.ord > 0 and up[Digit(c.ord - 1)] < down[Digit(c.ord - 1)]

repeat q:
    if read<1..=2>() == 1:
        let (a, b): (-1e9..=1e9, 0..=30 * q) = read()
        add(a, b)
    else:
        let k: 0..=30 * q = read()
        let d = Digit(k / 30)
        print(((up[d] ^ down[d]) >> (k % 30) & 1) ^ Int(borrows(d, (1 << (k % 30)) - 1)))
