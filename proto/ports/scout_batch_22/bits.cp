//! A 64-bit pattern that wraps, for generators a statement gives as C code.
//! `plan/11-values-and-types.md` has `U64` as a core type; the reference
//! interpreter has only the trapping `Int`, so this one is two 32-bit halves
//! (friction-backlog-batch-22-strings, X9).
let half = 1 << 32
let low = half - 1

/// An unsigned 64-bit number. `plus` wraps; shifts are by less than 32.
pub struct U64(hi: Int, lo: Int):
    /// From decimal digits, which may spell a number above `Int.MAX`.
    pub fn parse(s: Str) -> U64:
        var (hi, lo) = (0, 0)
        for c in s:
            lo = lo * 10 + (c - '0')
            hi = (hi * 10 + lo / half) & low
            lo = lo & low
        U64(hi, lo)

    pub fn xor(self, y: U64) -> U64 = U64(hi ^ y.hi, lo ^ y.lo)
    pub fn shl(self, k: Int) -> U64 = U64(((hi << k) | (lo >> (32 - k))) & low, (lo << k) & low)
    pub fn shr(self, k: Int) -> U64 = U64(hi >> k, ((hi << (32 - k)) & low) | (lo >> k))
    pub fn plus(self, y: U64) -> U64:
        let sum = lo + y.lo
        U64((hi + y.hi + sum / half) & low, sum & low)
    /// Bit `i`, counting from the lowest.
    pub fn bit(self, i: Int) -> Int = if i < 32: (lo >> i) & 1 else: (hi >> (i - 32)) & 1

@test
fn wraps_like_a_machine_word():
    let big = U64.parse("18446744073709551615")
    assert big == U64(low, low)
    assert big.plus(U64(0, 2)) == U64(0, 1)
    assert U64(1, 0).shr(1) == U64(0, 1 << 31)
    assert U64(0, 1 << 31).shl(1) == U64(1, 0)
    assert U64(1 << 31, 0).shl(1) == U64(0, 0)
    assert U64.parse("4294967296").bit(32) == 1
    assert U64(5, 3).xor(U64(1, 1)) == U64(4, 2)
