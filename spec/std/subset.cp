//! Subsets of a small domain as one machine word: the mask of mask DP with its domain kept.

/// A subset of a domain `B` of at most 64 members, stored as one word (bit `k` for `B`'s `k`-th
/// position). `B.subsets` is the domain of all `2^|B|` of them, so `[B.subsets]T` is a mask-DP
/// table, and `{}` is the empty subset. Every operation here is O(1) unless it says otherwise.
pub struct Subset<B>(bits: U64):
    /// The empty subset, also written `{}`.
    pub fn empty() -> Subset<B> = todo

    /// Every member of `B`.
    pub fn full() -> Subset<B> = todo

    /// `{b}`.
    pub fn single(b: B) -> Subset<B> = todo

    /// Whether `b` is a member; `b in s` calls it.
    pub fn contains(self, b: B) -> Bool = todo

    /// The number of members (a popcount).
    pub @get fn len(self) -> Int = todo

    /// `self` with `b` added.
    pub fn with(self, b: B) -> Subset<B> = todo

    /// `self` with `b` removed.
    pub fn without(self, b: B) -> Subset<B> = todo

    /// The members of `B` not in `self`.
    pub fn complement(self) -> Subset<B> = todo

    /// The least member, `None` for `{}`.
    pub fn first(self) -> ?B = todo

    /// The members in `B`'s order. O(1) per member.
    pub @get fn items(self) -> yields B = todo

    /// Every subset of `self`, `self` first and `{}` last. O(1) per subset.
    pub @get fn subsets(self) -> yields Subset<B> = todo

    /// Every superset of `self` within `B`, `self` first and `B` last. O(1) per subset.
    pub @get fn supersets(self) -> yields Subset<B> = todo

impl BitOr for Subset<B>:
    fn bitor(a, b) = todo

impl BitAnd for Subset<B>:
    fn bitand(a, b) = todo

impl BitXor for Subset<B>:
    fn bitxor(a, b) = todo

impl Sub for Subset<B>:
    fn sub(a, b) = todo
