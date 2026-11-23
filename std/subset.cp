//! Subsets of a small domain as one machine word: the mask of mask DP with its domain kept.

/// A subset of a domain `B` of at most 64 members, stored as one word (bit `k` for `B`'s `k`-th
/// position). `B.subsets` is the domain of all `2^|B|` of them, so `[B.subsets]T` is a mask-DP
/// table, and `Subset<B>.empty` is the empty subset (there is no `{}` literal). Every operation here is \(O(1)\) unless it says otherwise.
pub struct Subset<B>(bits: U64):
    /// The empty subset. There is no `{}` literal: `{}` is an empty block.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.empty.len == 0`
    @get pub fn empty() -> Subset<B> = Subset(U64(0))

    /// Every member of `B`: the word with its low `|B|` bits set.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.full.len == 4`
    @get pub fn full() -> Subset<B>:
        if B.count >= 64: return Subset(U64(0) - U64(1))
        Subset((U64(1) << B.count) - U64(1))

    /// `{b}`: the subset whose only member is `b`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.single(B(2)).items.collect() == [B(2)]`
    pub fn single(b: B) -> Subset<B> = Subset(U64(1) << b.ord)

    /// Whether `b` is a member; `b in s` calls it and `b not in s` negates it.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain B = 0..4; let s = Subset<B>.single(B(1)); B(1) in s and B(2) not in s`
    pub fn contains(self, b: B) -> Bool = (bits >> b.ord) & U64(1) == U64(1)

    /// The number of members (a popcount).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.single(B(0)).with(B(3)).len == 2`
    @get pub fn len(self) -> Int = popcount(bits)

    /// Whether the subset has no members.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain D = 0..4; Subset<D>.empty.is_empty and not Subset<D>.single(D(1)).is_empty`
    @get pub fn is_empty(self) -> Bool = bits == U64(0)

    /// `self` with `b` added. Adding a member already present returns `self` unchanged.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.empty.with(B(1)).with(B(1)) == Subset<B>.single(B(1))`
    pub fn with(self, b: B) -> Subset<B> = Subset(bits | (U64(1) << b.ord))

    /// `self` with `b` removed. Removing a non-member returns `self` unchanged.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.full.without(B(0)).len == 3`
    /// example: `domain B = 0..4; Subset<B>.empty.without(B(2)) == Subset<B>.empty`
    pub fn without(self, b: B) -> Subset<B> = Subset(bits & ((U64(1) << b.ord) ^ (U64(0) - U64(1))))

    /// Add `b` when `on`, remove it otherwise, in place (as `Bitset.set`).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain B = 0..4; var s = Subset<B>.empty; s.set(B(2), true); s.set(B(0), false); s == Subset<B>.single(B(2))`
    pub fn set(mut self, b: B, on: Bool):
        bits = if on: self.with(b).bits else: self.without(b).bits

    /// A new set with every member `k` places up (down for a negative `k`); members that leave `B`
    /// are dropped (as `Bitset.shifted`).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.single(B(1)).with(B(3)).shifted(1) == Subset<B>.single(B(2))`
    /// example: `domain B = 0..4; Subset<B>.single(B(1)).shifted(-2) == Subset<B>.empty`
    pub fn shifted(self, k: Int) -> Subset<B>:
        if k >= 64 or k <= -64: return Subset(U64(0))
        if k >= 0: return Subset((bits << k) & Subset<B>.full.bits)
        Subset(bits >> -k)

    /// The members of `B` not in `self`; no bit at or above `|B|` is ever set.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.single(B(0)).complement().len == 3`
    /// example: `domain B = 0..4; Subset<B>.empty.complement() == Subset<B>.full`
    pub fn complement(self) -> Subset<B> = Subset(bits ^ Subset<B>.full.bits)

    /// The least member in `B`'s order (the lowest set bit, by count-trailing-zeros), `None`
    /// for the empty subset. Peels one member per step in a mask DP: `s.first()` then `s.without(b)`.
    ///
    /// cost: \(O(1)\)
    /// empty: `None`
    /// ties: None possible; members are distinct, and the least in `B`'s order wins
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.single(B(3)).with(B(1)).first() == Some(B(1))`
    /// example: `domain B = 0..4; Subset<B>.empty.first() == None`
    pub fn first(self) -> ?B:
        if bits == U64(0): return None
        Some(B.nth(ctz(bits)))

    /// The members in `B`'s order; `for b in s` walks the same sequence.
    ///
    /// cost: \(O(1)\) per member
    /// empty: yields nothing
    /// order: increasing in `B`'s order (increasing bit index)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.single(B(3)).with(B(1)).items.collect() == [B(1), B(3)]`
    @get pub fn items(self) -> yields B:
        var m = bits
        while m != U64(0):
            yield B.nth(ctz(m))
            m &= m - U64(1)

    /// What `for b in s` walks: the members, as `s.items`.
    ///
    /// cost: \(O(1)\) per member
    /// empty: yields nothing
    /// order: increasing in `B`'s order (increasing bit index)
    /// traps: never
    /// example: `domain B = 0..4; [b for b in Subset<B>.single(B(3)).with(B(1))] == [B(1), B(3)]`
    pub fn each(self) -> yields B:
        for b in self.items: yield b

    /// Every subset of `self`, `self` first and the empty subset last: the submask walk
    /// `g = (g - 1) & self`, so the masks come in decreasing order as words. `2^s.len` subsets.
    ///
    /// cost: \(O(1)\) per subset, `2^s.len` subsets
    /// empty: `Subset<B>.empty.subsets` yields the empty subset once
    /// order: decreasing as words: `self` first, the empty subset last
    /// traps: never
    /// example: `domain B = 0..3; let s = Subset<B>.single(B(0)).with(B(1)); s.subsets.map(g => g.first()).collect() == [Some(B(0)), Some(B(1)), Some(B(0)), None]`
    @get pub fn subsets(self) -> yields Subset<B>:
        var g = bits
        while true:
            yield Subset(g)
            if g == U64(0): break
            g = (g - U64(1)) & bits

    /// Every superset of `self` within `B`, `self` first and `B` last: the walk
    /// `g = (g + 1) | self`, so the masks come in increasing order as words. `2^(|B| - s.len)`
    /// supersets.
    ///
    /// cost: \(O(1)\) per superset, `2^(|B| - s.len)` supersets
    /// empty: `Subset<B>.empty.supersets` yields every subset of `B`, the empty one first
    /// order: increasing as words: `self` first, `Subset<B>.full` last
    /// traps: never
    /// example: `domain B = 0..3; Subset<B>.single(B(1)).supersets.map(g => g.len).collect() == [1, 2, 2, 3]`
    @get pub fn supersets(self) -> yields Subset<B>:
        let all = Subset<B>.full.bits
        var g = bits
        while true:
            yield Subset(g)
            if g == all: break
            g = (g + U64(1)) | bits

impl BitOr for Subset<B>:
    fn bitor(a, b) = Subset(a.bits | b.bits)

impl BitAnd for Subset<B>:
    fn bitand(a, b) = Subset(a.bits & b.bits)

impl BitXor for Subset<B>:
    fn bitxor(a, b) = Subset(a.bits ^ b.bits)

impl Sub for Subset<B>:
    fn sub(a, b) = Subset(a.bits & (b.bits ^ (U64(0) - U64(1))))

impl Index for Subset<B>:
    let count = subset_count(B.count)
    @get fn ord(self) -> Int = Int(bits)
    fn nth(k: Int) -> Subset<B> = Subset(U64(k))

impl Into<Int> for Subset<B>:
    fn into(self) -> Int = Int(bits)

/// `2^n`, the number of subsets of `n` values, trapping in every build above 25 values.
fn subset_count(n: Int) -> Int:
    with checks(asserts: on):
        assert n <= 25, "Subset over {n} values: at most 25"
    1 << n
