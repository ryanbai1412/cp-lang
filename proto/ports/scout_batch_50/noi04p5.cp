// NOI 2004 P5, cut a 100-long rod into n main pieces and m parts in total
// original: corpus/submissions/3744270.cpp
// Both prices have one decimal, so the whole sum is exact in tenths.
let (pa, pb, n, m): (F64, F64, 1..=100, 1..=100) = read()
let (a, b) = (round(pa * 10), round(pb * 10))

// least sum of squares when `len` is cut into `parts` positive integers:
// the parts differ by at most one
fn even_split(len: Int, parts: Int) -> Int:
    let (q, r) = (len / parts, len % parts)
    r * (q + 1) ** 2 + (parts - r) * q ** 2

// the cheapest way to finish from position `at` with `pieces` main pieces and `parts` parts to go
@memo
fn finish(at: 0..=100, pieces: 0..=n, parts: 0..=m) -> ?Int:
    if pieces == 0: return if at == 100 and parts == 0: 0 else: none
    min(finish(at + len, pieces - 1, parts - p) + a * len ** 2 + b * even_split(len, p)
        for len in 1..=100 - at for p in 1..=min(len, parts))

print("{F64(finish(0, n, m)!) / 10:.1f}")
