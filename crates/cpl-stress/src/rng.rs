//! A small seeded generator (SplitMix64). The same seed gives the same
//! case on every machine, so a failing seed is a bug report.

#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value in `lo..=hi`. `lo` when the range is empty.
    pub fn range(&mut self, lo: i128, hi: i128) -> i128 {
        if hi <= lo {
            return lo;
        }
        let span = (hi - lo) as u128 + 1;
        let wide = (u128::from(self.next_u64()) << 64) | u128::from(self.next_u64());
        lo + (wide % span) as i128
    }

    /// True once in `n` calls, on average.
    pub fn one_in(&mut self, n: u64) -> bool {
        self.next_u64().is_multiple_of(n)
    }
}
