//! Subsets of a small domain as one machine word: the mask of mask DP with its domain kept.

/// A subset of a domain `B` of at most 64 members, stored as one word (bit `k` for `B`'s `k`-th
/// position). `B.subsets` is the domain of all `2^|B|` of them, so `[B.subsets]T` is a mask-DP
/// table, and `{}` is the empty subset. Every operation here is O(1) unless it says otherwise.
pub struct Subset<B>(bits: U64):
    /// The empty subset, also written `{}`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.empty().len == 0`
    pub fn empty() -> Subset<B> = todo

    /// Every member of `B`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.full().len == 4`
    pub fn full() -> Subset<B> = todo

    /// `{b}`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.single(B(2)).items.collect() == [2]`
    pub fn single(b: B) -> Subset<B> = todo

    /// Whether `b` is a member; `b in s` calls it.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; let s = Subset<B>.single(B(2)); B(2) in s and not s.contains(B(1))`
    pub fn contains(self, b: B) -> Bool = todo

    /// The number of members (a popcount).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.empty().with(B(0)).with(B(3)).len == 2`
    pub @get fn len(self) -> Int = todo

    /// `self` with `b` added; unchanged if `b` is already a member.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.single(B(1)).with(B(1)).len == 1`
    pub fn with(self, b: B) -> Subset<B> = todo

    /// `self` with `b` removed; unchanged if `b` is not a member.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.full().without(B(0)).items.collect() == [1, 2, 3]`
    pub fn without(self, b: B) -> Subset<B> = todo

    /// The members of `B` not in `self`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.single(B(0)).complement().items.collect() == [1, 2, 3]`
    pub fn complement(self) -> Subset<B> = todo

    /// The least member in `B`'s order, `None` for `{}`.
    ///
    /// cost: O(1)
    /// ties: none; members are distinct
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.empty().with(B(3)).with(B(1)).first() == Some(1)`
    /// example: `domain B = 0..4; Subset<B>.empty().first() == None`
    pub fn first(self) -> ?B = todo

    /// The members.
    ///
    /// cost: O(1) to make, O(1) per member
    /// order: `B`'s order
    /// traps: never
    /// example: `domain B = 0..4; Subset<B>.empty().with(B(3)).with(B(1)).items.collect() == [1, 3]`
    pub @get fn items(self) -> yields B = todo

    /// Every subset of `self`, by the `(t - 1) & self` walk.
    ///
    /// cost: O(1) per subset, `2^len` of them
    /// order: decreasing as masks: `self` first, `{}` last
    /// traps: never
    /// example: `domain B = 0..4; let s = Subset<B>.empty().with(B(0)).with(B(1)); [t.len for t in s.subsets] == [2, 1, 1, 0]`
    pub @get fn subsets(self) -> yields Subset<B> = todo

    /// Every superset of `self` within `B`, by the `(t + 1) | self` walk.
    ///
    /// cost: O(1) per subset, `2^(|B| - len)` of them
    /// order: increasing as masks: `self` first, `B` last
    /// traps: never
    /// example: `domain B = 0..3; let s = Subset<B>.single(B(0)); [t.len for t in s.supersets] == [1, 2, 2, 3]`
    pub @get fn supersets(self) -> yields Subset<B> = todo

impl BitOr for Subset<B>:
    fn bitor(a, b) = todo

impl BitAnd for Subset<B>:
    fn bitand(a, b) = todo

impl BitXor for Subset<B>:
    fn bitxor(a, b) = todo

impl Sub for Subset<B>:
    fn sub(a, b) = todo
