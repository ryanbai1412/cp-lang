// Blind Critic #4 — First Draft (written from docs/one-page-cheat-sheet.md v2.1 ONLY)
// Problem: ioi11p6 (Parrots — Two-Phase Combinatorial Encoding/Decoding)
// Original: corpus/submissions/3735613.cpp
// Exercises: I128, @memo over range types, Comb<P>() contrast (modular-only!), bit shifts, two-phase @grader

const BLOCK = 8
const SYMS = 40      // 5 * 8 symbols per 64-bit block
const ALPHA = 32     // 5-bit alphabet (0..32) per symbol; top 3 bits encode block id (0..8)

// Note on Comb<P>() (H22): Comb<P>() in cheat sheet §10 ONLY supports modular arithmetic Mod<P>,
// so exact stars-and-bars counts up to C(71, 40) ~ 2.1e20 cannot use Comb<P>()!
// Instead, exact I128 DP with @memo replaces C++'s custom 25-line overflow-flagged `struct Int`!
@memo
fn ways(rem: 0..=SYMS, lo: 0..=ALPHA) -> I128:
    if rem == 0:
        return 1
    if lo == ALPHA:
        return 0
    // Non-decreasing sequences of length `rem` with elements in `lo..ALPHA`:
    // either pick `lo` now (rem - 1 left from `lo`), or skip `lo` (rem left from `lo + 1`).
    return ways(rem - 1, lo) + ways(rem, lo + 1)

fn unrank(mut x: I128) -> List<Int>:
    var res: List<Int> = []
    var prev = 0
    for pos in 0..SYMS:
        let rem = SYMS - 1 - pos
        for v in prev..ALPHA:
            let cnt = ways(rem, v)
            if cnt > x:
                res.push(v)
                prev = v
                break
            x -= cnt
    return res

fn rank(seq: List<Int>) -> I128:
    var ans: I128 = 0
    var prev = 0
    for (i, val) in seq.items:
        let rem = SYMS - 1 - i
        for v in prev..val:
            ans += ways(rem, v)
        prev = val
    return ans

@grader
fn send(b: Int)

@grader
fn output(b: Int)

@grader
fn encode(n: Int, msg: List<Int>):
    // Process in chunks of 8 bytes; send 5 * chunk_len symbols per chunk
    var i = 0
    while i < n:
        let hi = min(n, i + BLOCK)
        let k = hi - i
        var packed: I128 = 0
        for j in i..hi:
            // BLIND-GUESS(H23): does `I128(msg[j])` exist and work with `<<`? Cheat sheet §1 only shows F64(x)/Int(ch)/Char(i)/Int(b)!
            packed |= I128(msg[j]) << ((j - i) * 8)
        let seq = unrank(packed)
        let block_id = i / BLOCK
        let send_cnt = 5 * k
        for j in (SYMS - send_cnt)..SYMS:
            send((block_id << 5) | seq[j])
        i += BLOCK

@grader
fn decode(n: Int, l: Int, encoded: List<Int>):
    let num_blocks = (n + BLOCK - 1) / BLOCK
    var buckets: [0..num_blocks]List<Int> = []
    for code in encoded:
        buckets[code >> 5].push(code & 31)

    for bi in 0..num_blocks:
        let start = bi * BLOCK
        let k = min(n, start + BLOCK) - start
        // Pad with leading zeros to SYMS (since List.insert(0, 0) isn't in §9, prepend via List.filled + sorted!)
        var seq = List.filled(SYMS - buckets[bi].len, 0) + buckets[bi]
        seq.sort()
        var packed = rank(seq)
        for _ in 0..k:
            // BLIND-GUESS(H23): `Int(packed & 255)` narrowing I128 -> Int
            output(Int(packed & 255))
            packed >>= 8
