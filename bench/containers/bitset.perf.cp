// Tuned: bitset.cp's knapsack and shift-or loop on raw 64-bit words, as
// bitset.cpp's std::bitset does it. `Bitset.shift_or` recounts every word after
// each shift to keep `len` O(1), which doubles the knapsack's work (perf PLAN.md).
const M = 200_001
const W = (M + 63) / 64
var weights: List<Int> = []
var s: U64 = U64(123456789)
for i in 0..4000:
    s = s * U64(6364136223846793005) + U64(1442695040888963407)
    weights.push(Int((s >> 33) % U64(100)) + 1)

fn trim(mut w: List<U64>):
    w[W - 1] &= (U64(1) << (M % 64)) - U64(1)

// w |= w << k, for 0 < k < M.
fn shift_or(mut w: List<U64>, k: Int):
    let (words, bits) = (k / 64, k % 64)
    unchecked:
        if bits == 0:
            for i in (words..W).rev: w[i] |= w[i - words]
        else:
            for i in (words + 1..W).rev:
                w[i] |= (w[i - words] << bits) | (w[i - words - 1] >> (64 - bits))
            w[words] |= w[0] << bits
    trim(mut w)

fn shifted_up(w: List<U64>, k: Int) -> List<U64>:
    var out = List.filled(W, U64(0))
    let (words, bits) = (k / 64, k % 64)
    unchecked:
        for i in words + 1..W:
            out[i] = (w[i - words] << bits) | (w[i - words - 1] >> (64 - bits))
        out[words] = w[0] << bits
    trim(mut out)
    out

fn shifted_down(w: List<U64>, k: Int) -> List<U64>:
    var out = List.filled(W, U64(0))
    unchecked:
        for i in 0..W - 1: out[i] = (w[i] >> k) | (w[i + 1] << (64 - k))
        out[W - 1] = w[W - 1] >> k
    out

var cnt = List.filled(M, 0)
for w in weights:
    if w in 1..M: cnt[w] += 1
var reach = List.filled(W, U64(0))
reach[0] = U64(1)
for w in 1..M:
    var c = cnt[w]
    var k = 1
    while c > 0:
        let take = min(k, c)
        if take * w < M: shift_or(mut reach, take * w)
        c -= take
        k *= 2

let b = shifted_up(reach, 37)
let c = shifted_down(reach, 19)
var ans = sum(popcount(x) for x in reach)
var a = reach
for step in 0..200:
    shift_or(mut a, step % 31 + 1)
    unchecked:
        ans += sum(popcount(a[i] & b[i] & c[i]) for i in 0..W)
print(ans)
