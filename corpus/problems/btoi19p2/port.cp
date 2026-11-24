// Source: Baltic OI 2019 P2 (Nautilus), https://dmoj.ca/problem/btoi19p2
// limits: 1000 ms, 256 MB
// Problem: an R x C map (R, C <= 500) of water '.' and island '#', and M <= 5000 signals, each
//   N, E, S, W or '?' (any of the four). A submarine started on some water cell and made one move
//   per signal, always on water. Print how many cells it can be in now.
// Solution: keep the set of possible cells as bits, 30 columns per word: a signal shifts every
//   row's words one step (OR of all four for '?') and masks them with the water bits, then count
//   the bits at the end. O(R C M / 30).
// original: corpus/submissions/4456161.cpp
// Bounds: r, c: 1..=500; m: 1..=5000
let r: 1..=500 = read()
let c: 1..=500 = read()
let m: 1..=5000 = read()
domain Row = 0..r
domain Col = 0..c
let map: [Row][Col]Char = read()
let signals: [m]Char = read()

// 30 columns per word: the prototype emits `1 << k` as a 32-bit shift, so a
// word must stay clear of bit 31.
const B = 30
domain Word = 0..ceil_div(c, B)
let full = (1 << B) - 1

fn sea_bits() -> [Row, Word]Int:
    var s: [Row, Word]Int = 0
    for i in Row:
        for j in Col:
            if map[i][j] == '.': s[i, Word(j.idx / B)] |= 1 << (j.idx % B)
    s

let sea = sea_bits()

fn step(at: [Row, Word]Int, sig: Char) -> [Row, Word]Int:
    var to: [Row, Word]Int = 0
    for i in Row:
        for w in Word:
            var x = 0
            if sig == 'N' or sig == '?':
                if let below = i.next: x |= at[below, w]
            if sig == 'S' or sig == '?':
                if let above = i.prev: x |= at[above, w]
            if sig == 'E' or sig == '?':
                x |= (at[i, w] << 1) & full
                if let p = w.prev: x |= at[i, p] >> (B - 1)
            if sig == 'W' or sig == '?':
                x |= at[i, w] >> 1
                if let q = w.next: x |= (at[i, q] & 1) << (B - 1)
            to[i, w] = x & sea[i, w]
    to

// The prototype has no popcount intrinsic yet.
fn ones(x0: Int) -> Int:
    var x = x0
    var k = 0
    while x != 0:
        x &= x - 1
        k += 1
    k

var at = sea
for sig in signals: at = step(at, sig)
print(sum(ones(at[i, w]) for i in Row for w in Word))
