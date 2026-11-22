//! Subsets of a small domain as one machine word: the mask of mask DP with its domain kept.

/// A subset of a domain `B` of at most 64 members, stored as one word (bit `k` for `B`'s `k`-th
/// position). `B.subsets` is the domain of all `2^|B|` of them, so `[B.subsets]T` is a mask-DP
/// table, and `{}` is the empty subset. Every operation here is O(1) unless it says otherwise.
pub struct Subset<B>(bits: U64):
    /// The empty subset, also written `{}`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.empty.len == 0`
    pub @get fn empty() -> Subset<B> = todo

    /// Every member of `B`: the word with its low `|B|` bits set.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.full.len == 4`
    pub @get fn full() -> Subset<B> = todo

    /// `{b}`: the subset whose only member is `b`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.single(B(2)).items.collect() == [B(2)]`
    pub fn single(b: B) -> Subset<B> = todo

    /// Whether `b` is a member; `b in s` calls it and `b not in s` negates it.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; let s = Subset<B>.single(B(1)); B(1) in s and B(2) not in s`
    pub fn contains(self, b: B) -> Bool = todo

    /// The number of members (a popcount).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.single(B(0)).with(B(3)).len == 2`
    pub @get fn len(self) -> Int = todo

    /// `self` with `b` added. Adding a member already present returns `self` unchanged.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.empty.with(B(1)).with(B(1)) == Subset<B>.single(B(1))`
    pub fn with(self, b: B) -> Subset<B> = todo

    /// `self` with `b` removed. Removing a non-member returns `self` unchanged.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.full.without(B(0)).len == 3`
    /// example: `domain B = 0..4; Subset<B>.empty.without(B(2)) == Subset<B>.empty`
    pub fn without(self, b: B) -> Subset<B> = todo

    /// Add `b` when `on`, remove it otherwise, in place (as `Bitset.set`).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; var s = Subset<B>.empty; s.set(B(2), true); s.set(B(0), false); s == Subset<B>.single(B(2))`
    pub fn set(mut self, b: B, on: Bool) = todo

    /// A new set with every member `k` places up (down for a negative `k`); members that leave `B`
    /// are dropped (as `Bitset.shifted`).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.single(B(1)).with(B(3)).shifted(1) == Subset<B>.single(B(2))`
    /// example: `domain B = 0..4; Subset<B>.single(B(1)).shifted(-2) == Subset<B>.empty`
    pub fn shifted(self, k: Int) -> Subset<B> = todo

    /// The members of `B` not in `self`; no bit at or above `|B|` is ever set.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.single(B(0)).complement().len == 3`
    /// example: `domain B = 0..4; Subset<B>.empty.complement() == Subset<B>.full`
    pub fn complement(self) -> Subset<B> = todo

    /// The least member in `B`'s order (the lowest set bit, by count-trailing-zeros), `None`
    /// for `{}`. Peels one member per step in a mask DP: `s.first()` then `s.without(b)`.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: none possible; members are distinct, and the least in `B`'s order wins
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.single(B(3)).with(B(1)).first() == Some(B(1))`
    /// example: `domain B = 0..4; Subset<B>.empty.first() == None`
    pub fn first(self) -> ?B = todo

    /// The members in `B`'s order; `for b in s` walks the same sequence.
    ///
    /// cost: O(1) per member
    /// empty: yields nothing
    /// order: increasing in `B`'s order (increasing bit index)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.single(B(3)).with(B(1)).items.collect() == [B(1), B(3)]`
    pub @get fn items(self) -> yields B = todo

    /// Every subset of `self`, `self` first and `{}` last: the submask walk
    /// `g = (g - 1) & self`, so the masks come in decreasing order as words. `2^s.len` subsets.
    ///
    /// cost: O(1) per subset, `2^s.len` subsets
    /// empty: `{}.subsets` yields `{}` once
    /// order: decreasing as words: `self` first, `{}` last
    /// traps: never
    /// example: `domain B = 0..3; let s = Subset<B>.single(B(0)).with(B(1)); s.subsets.map(g => g.first()).collect() == [Some(B(0)), Some(B(1)), Some(B(0)), None]`
    pub @get fn subsets(self) -> yields Subset<B> = todo

    /// Every superset of `self` within `B`, `self` first and `B` last: the walk
    /// `g = (g + 1) | self`, so the masks come in increasing order as words. `2^(|B| - s.len)`
    /// supersets.
    ///
    /// cost: O(1) per superset, `2^(|B| - s.len)` supersets
    /// empty: `{}.supersets` yields every subset of `B`, `{}` first
    /// order: increasing as words: `self` first, `Subset<B>.full` last
    /// traps: never
    /// example: `domain B = 0..3; Subset<B>.single(B(1)).supersets.map(g => g.len).collect() == [1, 2, 2, 3]`
    pub @get fn supersets(self) -> yields Subset<B> = todo

impl BitOr for Subset<B>:
    fn bitor(a, b) = todo

impl BitAnd for Subset<B>:
    fn bitand(a, b) = todo

impl BitXor for Subset<B>:
    fn bitxor(a, b) = todo

impl Sub for Subset<B>:
    fn sub(a, b) = todo
