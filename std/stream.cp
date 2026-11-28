//! The program's one random stream, the same on every engine and build. Core re-exports `rand`,
//! so no module imports it; `std.rand` builds `shuffle`, `gen` and `Rng` on top.
//!
//! start: `rand`
//! example: `let x = rand(1, 6); 1 <= x and x <= 6`

fn check_range(ok: Bool) = @intrinsic("range_check")
fn rand_word() -> U64 = @intrinsic("rand_word")

/// `rand(lo, hi)`: an `Int` in `lo..=hi` drawn from the program's one random stream, the same
/// on every engine and build (R02.10.1).
pub fn rand(lo: Int, hi: Int) -> Int:
    check_range(lo <= hi)
    // `hi - lo + 1` in bits, which is `0` for the whole of `Int`.
    let span = U64(hi) - U64(lo) + U64(1)
    let r = if span == U64(0): rand_word() else: rand_word() % span
    let half = Int(r / U64(2))
    lo + half + half + Int(r % U64(2))
