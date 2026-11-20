// IOI '11 P6 — Parrots (runnable proto/cpc port of ioi11p6_draft.cp)
// Prototype deltas vs v2.1 draft: no `@grader fn`, no `I128`/`U64`, no lexically scoped `@memo`.
// Uses 2-limb base-2^32 big-int arithmetic in signed 64-bit `Int` to rank/unrank exact 8-byte blocks
// into 40 symbols over a 32-value alphabet (tagged with 3-bit block id => symbols in 0..255, m <= 5*n).

let BASE = 4294967296
let MAX_LEN = 40
let ALPHA = 32

// Precompute dp[len][lo] = number of non-decreasing sequences of `len` symbols from `lo..31`.
// Stored as saturating 2-limb base-2^32 integers `(hi, lo)` + `ovf` flag (> 2^64 - 1).
domain RL = 0..=MAX_LEN
domain AL = 0..ALPHA
var dp_hi: [RL, AL]Int = 0
var dp_lo: [RL, AL]Int = 0
var dp_ovf: [RL, AL]Bool = false

for j in 0..ALPHA:
    dp_lo[RL.from_idx(0), AL.from_idx(j)] = 1

for i in 1..=MAX_LEN:
    let ri = RL.from_idx(i)
    let rp = RL.from_idx(i - 1)
    for j in 0..ALPHA:
        let aj = AL.from_idx(j)
        var s_hi = 0
        var s_lo = 0
        var s_ovf = false
        for k in j..ALPHA:
            let ak = AL.from_idx(k)
            if dp_ovf[rp, ak]:
                s_ovf = true
            if not s_ovf:
                s_lo += dp_lo[rp, ak]
                s_hi += dp_hi[rp, ak] + s_lo / BASE
                s_lo = s_lo % BASE
                if s_hi >= BASE:
                    s_ovf = true
        dp_hi[ri, aj] = s_hi
        dp_lo[ri, aj] = s_lo
        dp_ovf[ri, aj] = s_ovf

fn dp_gt(len_i: Int, sym: Int, x_hi: Int, x_lo: Int) -> Bool:
    let ri = RL.from_idx(len_i)
    let aj = AL.from_idx(sym)
    if dp_ovf[ri, aj]:
        return true
    let dh = dp_hi[ri, aj]
    if dh != x_hi:
        return dh > x_hi
    return dp_lo[ri, aj] > x_lo

// Read n bytes, encode to <= 5*n symbols in 0..255, adversarially shuffle/reverse, then decode!
let n: 1..=64 = read()
var msg: List<Int> = []
for _ in 0..n:
    let b: Int = read()
    msg.push(b)

// Encode
var sent: List<Int> = []
var blk = 0
while blk * 8 < n:
    let start = blk * 8
    let end = if start + 8 < n: start + 8 else: n
    let cnt = end - start
    var x_lo = 0
    var x_hi = 0
    var mul = 1
    for j in 0..cnt:
        let b = msg[start + j]
        if j < 4:
            x_lo += b * mul
            mul *= 256
        else:
            if j == 4:
                mul = 1
            x_hi += b * mul
            mul *= 256

    var seq: List<Int> = []
    var prev = 0
    for step in 0..MAX_LEN:
        let rem = MAX_LEN - 1 - step
        var chosen = ALPHA - 1
        for j in prev..ALPHA:
            if dp_gt(rem, j, x_hi, x_lo):
                chosen = j
                break
            let ri = RL.from_idx(rem)
            let aj = AL.from_idx(j)
            x_lo -= dp_lo[ri, aj]
            x_hi -= dp_hi[ri, aj]
            if x_lo < 0:
                x_lo += BASE
                x_hi -= 1
        seq.push(chosen)
        prev = chosen

    let take = 5 * cnt
    for j in MAX_LEN - take..MAX_LEN:
        sent.push(blk * 32 + seq[j])
    blk += 1

// Adversarial channel: reverse sent array (destroys order completely; decoder must be order-invariant).
sent.rev()

// Verify channel constraints (m <= 5*n and every symbol in 0..255).
assert sent.len <= 5 * n
for s in sent:
    assert s >= 0 and s <= 255

// Decode from unordered `sent`.
domain B = 0..8
var buckets: [B]List<Int> = []
for s in sent:
    let bi = s / 32
    let sym = s % 32
    buckets[B.from_idx(bi)].push(sym)

var decoded: List<Int> = []
blk = 0
while blk * 8 < n:
    let start = blk * 8
    let end = if start + 8 < n: start + 8 else: n
    let cnt = end - start
    var cm: List<Int> = []
    let raw = buckets[B.from_idx(blk)]
    for _ in 0..MAX_LEN - raw.len:
        cm.push(0)
    for x in raw:
        cm.push(x)
    cm.sort()

    var x_hi = 0
    var x_lo = 0
    for i in 0..MAX_LEN:
        let rem = MAX_LEN - 1 - i
        let lo_j = if i > 0: cm[i - 1] else: 0
        for j in lo_j..cm[i]:
            let ri = RL.from_idx(rem)
            let aj = AL.from_idx(j)
            x_lo += dp_lo[ri, aj]
            x_hi += dp_hi[ri, aj] + x_lo / BASE
            x_lo = x_lo % BASE

    for j in 0..cnt:
        if j < 4:
            decoded.push(x_lo % 256)
            x_lo /= 256
        else:
            decoded.push(x_hi % 256)
            x_hi /= 256
    blk += 1

print(sent.len)
for b in decoded:
    print(b)
