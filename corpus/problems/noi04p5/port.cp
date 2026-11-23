// NOI 2004 P5, cut a 100-long rod into n main pieces and m parts in total (buildable today-form)
// original: corpus/submissions/3744270.cpp
// The two prices have one decimal, so everything is counted in tenths.
// Each price is read as two Ints, `whole` then `digit`: the prototype has no
// F64, a Str cannot be indexed, and its Int reader swallows the one byte after
// a number, here the decimal point. So the input must spell out one decimal.
fn read_tenths() -> Int:
    let whole: 0..=1e6 = read()
    let frac: 0..=9 = read()
    whole * 10 + frac

let a = read_tenths()
let b = read_tenths()
let n: 1..=100 = read()
let m: 1..=100 = read()

// least sum of squares when `len` is cut into `parts` positive integers:
// the parts differ by at most one
fn even_split(len: Int, parts: Int) -> Int:
    let (q, r) = (len / parts, len % parts)
    r * (q + 1) * (q + 1) + (parts - r) * q * q

// cost[i, k, j]: i main pieces cover the first k units and hold j parts
var cost: [0..=n, 0..=100, 0..=m]?Int = None
cost[0, 0, 0] = 0
for i in 0..n:
    for k in 0..100:
        for j in 0..m:
            if let base = cost[i, k, j]:
                for len in 1..=100 - k:
                    for parts in 1..=min(len, m - j):
                        cost[
                            i + 1,
                            k + len,
                            j + parts,
                        ] min= base + a * len * len + b * even_split(len, parts)
let best = cost[n, 100, m]!
// no string interpolation in the prototype
print(best / 10, end: "")
print(".", end: "")
print(best % 10)
