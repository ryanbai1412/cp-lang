//! String algorithms. They only compare elements, so they work on a `Str`
//! (ASCII bytes) and on any `List<T>` alike. Positions are 0-based.

use std.seq.{reverse}

/// O(|self| * n). `s` written `n` times over: `"0".repeat(4)` is `"0000"`.
pub fn repeat(self: Str, n: Int) -> Str = todo

/// O(|pat|). True when `self` opens with `pat` (`Str`, `List<T>`, `[D]T`, or views).
@allow(unproven_list_index)
pub fn starts_with<S: Indexed, P: Indexed>(self: S, pat: P) -> Bool = todo

/// O(|pat|). True when `self` closes with `pat` (`Str`, `List<T>`, `[D]T`, or views).
@allow(unproven_list_index)
pub fn ends_with<S: Indexed, P: Indexed>(self: S, pat: P) -> Bool = todo

/// O(|self| + |p|). Where `p` first stands in `s`, or `none`: `"banana".find("nan")` is `2`. A key of `s`, so on a
/// slice it is valid on the source: `s[l..].find(c)` needs no `+ l`. `p` is a `Str` or
/// one `Char`: `"banana".find('n')` is `2`. O(s.len + p.len).
pub fn find<P>(self: Str, p: P) -> ?Int = todo

/// How many times `c` stands in `s`: `"banana".count('a')` is `3`. O(s.len).
pub fn count(self: Str, c: Char) -> Int = todo

/// O(|self| + |p|). Where `p` last stands in `s`, or `none`: `"banana".rfind("an")` is `3`. `p` is a `Str` or
/// one `Char`: `"banana".rfind('a')` is `5`. O(s.len + p.len).
pub fn rfind<P>(self: Str, p: P) -> ?Int = todo

/// O(1). The small letter of a capital, and any other character as it is: `'A'.lower` is `'a'`.
pub @get fn lower(self: Char) -> Char = todo

/// O(1). The capital of a small letter, and any other character as it is: `'a'.upper` is `'A'`.
pub @get fn upper(self: Char) -> Char = todo

/// O(1). The value of a decimal digit: `'7'.digit` is 7. Stops on any other character.
pub @get fn digit(self: Char) -> Int = todo

/// O(1). True when `self` is an ASCII decimal digit (`'0'..='9'`).
pub @get fn is_digit(self: Char) -> Bool = todo

/// O(1). True when `self` is an ASCII lowercase letter (`'a'..='z'`).
pub @get fn is_lower(self: Char) -> Bool = todo

/// O(1). True when `self` is an ASCII uppercase letter (`'A'..='Z'`).
pub @get fn is_upper(self: Char) -> Bool = todo

/// O(1). True when `self` is an ASCII letter (`'a'..='z'` or `'A'..='Z'`).
pub @get fn is_alpha(self: Char) -> Bool = todo

/// O(1). True when `self` is an ASCII letter or decimal digit.
pub @get fn is_alnum(self: Char) -> Bool = todo

/// O(1). True when `self` is ASCII whitespace (`' '`, `'\t'`, `'\n'`, `'\r'`).
pub @get fn is_space(self: Char) -> Bool = todo

/// O(|self|). Lowercase copy of `self`.
pub fn to_lower(self: Str) -> Str = todo

/// O(|self|). Uppercase copy of `self`.
pub fn to_upper(self: Str) -> Str = todo

/// O(|self|). Copy of `self` with leading and trailing ASCII whitespace removed.
pub fn trim(self: Str) -> Str = todo

/// O(|self|). Split `self` at every occurrence of `sep`.
pub fn split(self: Str, sep: Char) -> List<Str> = todo

/// O(|self|). Split `self` at the first occurrence of `sep` into `(before, after)`, or `none` if absent.
pub fn split_first(self: Str, sep: Char) -> ?(Str, Str) = todo

/// O(|self|). Split `self` into non-empty runs separated by ASCII whitespace.
pub fn split_whitespace(self: Str) -> List<Str> = todo

/// O(total length). Join `parts` with `self` as separator: `", ".join(xs)`.
pub fn join<S: Seq>(self: Str, parts: S) -> Str = todo

/// O(|self| + |out|). Replace every non-overlapping occurrence of `from` with `to`.
pub fn replace(self: Str, from: Str, to: Str) -> Str = todo

/// O(|self|). Parse a signed integer in `base` (2..=36; digits past 9 are letters in either
/// case), or `none` if malformed or out of `Int` range. `"ff".parse_int(base: 16) == some(255)`.
pub fn parse_int(self: Str, base: Int = 10) -> ?Int = todo

/// O(|self|). `self` read as a `T` by the same rules as `read<T>()` (core 10.2), or `none` if a
/// token is malformed, out of `T`'s range, or left over: `"-12".parse<Mod<7>>() == some(2)`.
pub fn parse<T: Read>(self: Str) -> ?T = todo

/// O(n). `z[i]` is the length of the longest common prefix of `s` and `s[i..]`; `z[0]` is `s.len`.
/// Keyed by `s.keys`, so `z_function(s[l..])[i]` takes `i` from the slice (2b).
/// Works on any `Indexed` sequence (`Str`, `List<T>`, `[D]T`, `Range<I>`, views) without conversion. O(n).
@allow(unproven_list_index)
pub fn z_function<S: Indexed, K>(s: S) -> [K]Int = todo

/// O(n). `p[i]` is the length of the longest proper prefix of `s[..=i]` that is also its suffix.
/// Works on any `Indexed` sequence (`Str`, `List<T>`, `[D]T`, `Range<I>`, views) without conversion. O(n).
@allow(unproven_list_index)
pub fn prefix_function<S: Indexed, K>(s: S) -> [K]Int = todo

/// O(|self| + |pat|). The start of every occurrence of `pat` in `self`: `s.find_all(p)`, in increasing order (Knuth-Morris-Pratt).
/// Works on any `Indexed` sequences (`Str`, `List<T>`, `[D]T`, views) without conversion.
/// O(text.len + pat.len). An empty pattern occurs at every position, including the end.
@allow(unproven_list_index)
pub fn find_all<S: Indexed, P: Indexed, K>(self: S, pat: P) -> yields K = todo

/// Length of the longest common prefix of `self` and `other`. Works on any `Indexed` sequences. O(min(|self|, |other|)).
@allow(unproven_list_index)
pub fn common_prefix_len<A: Indexed, B: Indexed>(self: A, other: B) -> Int = todo

/// Length of the longest common suffix of `self` and `other`. Works on any `Indexed` sequences. O(min(|self|, |other|)).
@allow(unproven_list_index)
pub fn common_suffix_len<A: Indexed, B: Indexed>(self: A, other: B) -> Int = todo

/// The length of the shortest period of `s`: the smallest `d >= 1` with `s[i] == s[i + d]` for all `i`. O(n).
pub fn period<S: Indexed>(s: S) -> Int = todo

/// True when `s` reads the same forwards and backwards. Works on any `Indexed` sequence. O(n).
@allow(unproven_list_index)
pub fn is_palindrome<S: Indexed>(s: S) -> Bool = todo

/// All non-empty proper border lengths of `s`, longest first. Works on any `Indexed` sequence. O(n).
@allow(unproven_list_index)
pub fn borders<S: Indexed>(s: S) -> yields Int = todo

/// O(|self| + |pat|). `res[i]` is the length of the longest prefix of `pat` ending at `self[i]` (KMP).
/// Works on any `Indexed` sequences. O(|self| + |pat|).
@allow(unproven_list_index)
pub fn prefix_matches<S: Indexed, P: Indexed, K>(self: S, pat: P) -> [K]Int = todo

/// O(|a| + |b|). Length of the longest suffix of `a` that is a prefix of `b` (at most `min(|a|, |b|)`).
/// Works on any `Indexed` sequences. O(|a| + |b|).
@allow(unproven_list_index)
pub fn overlap<A: Indexed, B: Indexed>(a: A, b: B) -> Int = todo

/// O(n). Start indices of the unique non-increasing Lyndon factorization of `s` (Duval).
/// Works on any `Indexed` sequence. O(n).
@allow(unproven_list_index)
pub fn lyndon_factorization<S: Indexed, K>(s: S) -> List<K> = todo

/// O(n). Start index of the lexicographically least cyclic rotation of `s` (Duval/Booth).
/// Ties go to the smallest key; `none` for an empty sequence. Works on any `Indexed` sequence. O(n).
@allow(unproven_list_index)
pub fn min_rotation<S: Indexed, K>(s: S) -> ?K = todo

/// O(n). Start index of the lexicographically greatest cyclic rotation of `s` (Duval/Booth).
/// Ties go to the smallest key; `none` for an empty sequence.
pub fn max_rotation<S: Indexed, K>(s: S) -> ?K = todo

/// O(|pat| * |A|). Prefix (KMP) automaton over alphabet domain `A`, for O(1) transitions in DP.
pub struct KmpAutomaton<A>:
    alpha: 0..=2_000_000_000
    goto: List<0..=2_000_000_000>
    pi: List<0..=2_000_000_000>

    /// O(|pat| * |A|). Build the KMP transition table for `pat` over alphabet `A`.
    pub fn new<S: Seq>(pat: S) -> KmpAutomaton<A> = todo

    /// O(1). Number of states (`|pat| + 1`, states `0..=|pat|`).
    pub @get fn len(self) -> Int = todo

    /// O(1). Next matched prefix length after reading `a` from `state`.
    pub fn step(self, state: Int, a: A) -> Int = todo

    /// O(1). Failure link of `state` (`0` at state `0`).
    pub fn link(self, state: Int) -> Int = todo

    /// O(|text|). Yield every start index where `pat` occurs in `text`.
    pub fn find_all<S: Seq>(self, text: S) -> yields Int = todo

/// O(|self| * ceil(|pat| / 64)). Bitset (Shift-And) matching; yields start indices of `pat` in `self`.
pub fn shift_and<S: Indexed, P: Indexed>(self: S, pat: P) -> yields Int = todo

/// O(|self| * ceil(|pat| / 64)). Yield start indices where `pat` matches `self`, with `wildcard` matching any element.
pub fn find_all_wildcard<S: Indexed, P: Indexed>(
    self: S,
    pat: P,
    wildcard: Char = '?',
) -> yields Int = todo

/// O(n) space, O(1) queries. Manacher palindrome radii. `odd[i]` is the number of palindromes centred on `s[i]`;
/// `even[i]` is the number centred between `s[i - 1]` and `s[i]`.
pub struct Palindromes(odd: List<Int>, even: List<Int>):
    /// O(1). Length of the indexed sequence.
    pub @get fn len(self) -> Int = todo

    /// O(1). True when the indexed sequence is empty.
    pub @get fn is_empty(self) -> Bool = todo

    /// O(1). Length of the longest odd-length palindrome centred on position `i`.
    @allow(unproven_list_index)
    pub fn longest_at(self, i: Int) -> Int = todo

    /// O(1). Length of the longest even-length palindrome centred on cut `i` (between `i - 1` and `i`).
    @allow(unproven_list_index)
    pub fn longest_between(self, i: Int) -> Int = todo

    /// True when the range `r` (`l..r`, `l..=r`, or `Range<D>`) is a palindrome. O(1).
    pub fn is_palindrome<R>(self, r: R) -> Bool = todo

    /// Total number of non-empty palindromic substrings. O(n).
    pub fn count(self) -> Int = todo

    /// Longest palindromic substring range `(l, r)` (half-open); ties go to the leftmost. O(n).
    pub fn longest(self) -> (l: Int, r: Int) = todo

/// Linear-time palindrome radii (Manacher's algorithm). Works on any `Indexed` sequence. O(n).
@allow(unproven_list_index)
pub fn manacher<S: Indexed>(s: S) -> Palindromes = todo

fn induced_sort(
    vec: List<Int>,
    counts: List<Int>,
    is_l: List<Bool>,
    lms: List<Int>,
    mut sa: List<Int>,
) = todo

@allow(unproven_list_index)
fn sa_is(vec: List<Int>, alpha: Int) -> List<Int> = todo

/// O(n) SA-IS + Kasai LCP (O(n log n) with RMQ). Suffix array, inverse rank, adjacent LCP, and O(1) LCP/comparison queries.
pub struct SuffixArray:
    sa: List<Int>
    rank: List<Int>
    lcp: List<Int>

    /// O(n). Build over any indexed sequence of `Char` or integers: `SuffixArray(s)`.
    pub fn new<S: Indexed>(s: S) -> SuffixArray = todo

    /// O(1). Length of the indexed sequence.
    pub @get fn len(self) -> Int = todo

    /// O(1). True when the indexed sequence is empty.
    pub @get fn is_empty(self) -> Bool = todo

    /// O(1). Longest common prefix of suffixes `s[i..]` and `s[j..]`.
    pub fn lcp_between(self, i: Int, j: Int) -> Int = todo

    /// O(|pat| log n). Rank range `lo..hi` in `sa` of all suffixes starting with `pat`.
    pub fn rank_range<P: Indexed>(self, pat: P) -> Range<Int> = todo

    /// O(|pat| log n + matches). Yield every start index where `pat` occurs in `s` (in suffix-array order).
    pub fn find_all<P: Indexed>(self, pat: P) -> yields Int = todo

    /// O(|pat| log n). Number of occurrences of `pat` in `s`.
    pub fn count<P: Indexed>(self, pat: P) -> Int = todo

    /// O(log n). Rank range `lo..hi` in `sa` of all suffixes whose LCP with `s[i..]` is at least `len`.
    pub fn lcp_range(self, i: Int, len: Int) -> Range<Int> = todo

    /// O(1). Lexicographic comparison of `s[l1..r1]` and `s[l2..r2]`: returns `-1`, `0`, or `1`.
    pub fn compare(self, l1: Int, r1: Int, l2: Int, r2: Int) -> Int = todo

    /// O(n). Number of distinct non-empty substrings of `s`.
    pub fn distinct(self) -> Int = todo

    /// O(1). Minimum adjacent LCP across the rank range `r` (`r.start..r.end`).
    pub fn lcp_of_ranks(self, r: Range<Int>) -> Int = todo

    /// O(1). Rank index `k in r` achieving the minimum adjacent LCP (for suffix-tree divide-and-conquer).
    pub fn split(self, r: Range<Int>) -> Int = todo

    /// O(log n). The `k`-th lexicographically smallest distinct non-empty substring (0-based), or `none`.
    pub fn kth_distinct(self, k: Int) -> ?Range<Int> = todo

/// O(n) for `Str` or small alphabets, O(n log n) for general `Ord`. Build the suffix array (`sa`, `rank`, `lcp`, and RMQ) for `s`.
pub fn suffix_array<S: Indexed>(s: S) -> SuffixArray = todo

/// O(1) value. One edge of the compressed suffix tree of a sequence, built in O(n) from SA + LCP.
pub struct SuffixTreeEdge(
    top_size: Int,
    bottom_size: Int,
    parent_len: Int,
    edge_len: Int,
    rank_range: Range<Int>,
)

/// O(n). All edges of the compressed suffix tree of `s` (bottom-up from SA + LCP).
pub fn suffix_tree_edges<S: Indexed>(s: S) -> List<SuffixTreeEdge> = todo

/// O(n log n). All maximal repetitions (runs) of `s` via Bannai's Lyndon-runs theorem.
/// Each `(l, r, period)` is a half-open range `s[l..r]` with minimal period `period`,
/// length `r - l >= 2 * period`, that cannot be extended left or right with period `period`.
/// Returned sorted by `(l, r)`. O(n log n) time and memory.
@allow(unproven_list_index)
pub fn lyndon_runs<S: Indexed>(s: S) -> List<(l: Int, r: Int, period: Int)> = todo

// ---- generalized rolling hash

const RH_M1 = 1_000_000_007
const RH_M2 = 1_000_000_009

/// O(1) space and comparison. A 64-bit double-mod polynomial hash value (`Copy + Eq + Ord + Hash`).
/// Can itself be fed into another `RollingHash` (e.g. hashing rows into a column hash for 2D grids).
pub struct HashVal(h1: Int, h2: Int):
    /// O(1). Mix both 31-bit halves into one integer token so a `RollingHash` can hash `[D]HashVal`.
    pub @get fn hash_token(self) -> Int = todo

/// O(1) per element. Elements accepted by `RollingHash`: `Char`, `Int`, `Bool`, `HashVal`, or any user type
/// implementing `hash_token`.
pub trait Hashable:
    @get fn hash_token(self) -> Int = todo

@get fn ord(self: Int) -> Int = todo

/// O(n) build, O(1) slice hash. Generalized double-mod polynomial rolling hash over `Str`, `List<T>`, or any `[D]T`
/// (`Char`, `Int`, `Bool`, `HashVal`).
///
/// Built in O(n); substring queries `.get(l..r)` / `.slice(l, r)` and `.combine(a, b, b_len)`
/// are O(1). Each constructor draws independent random salted bases unless `bases:` is passed
/// explicitly, so 2D row and column hashes never collide on transposes.
pub struct RollingHash:
    bases: (Int, Int)
    p1: List<Int>
    p2: List<Int>
    pow1: List<Int>
    pow2: List<Int>

    /// Build a double-modulus polynomial rolling hash over `seq`. O(n).
    @allow(unproven_list_index)
    pub fn new<S: Seq>(seq: S, bases: ?(Int, Int) = none) -> RollingHash = todo

    /// O(1) amortized. Append one element `x` to the hashed sequence.
    pub fn push<T: Hashable>(mut self, x: T) = todo

    /// O(1). Remove the last element from the hashed sequence.
    pub fn pop(mut self) = todo

    /// O(|seq|). Hash `seq` directly under `self.bases` without building a full `RollingHash`.
    pub fn hash_of<S: Seq>(self, seq: S) -> HashVal = todo

    /// O(|parts|). Concatenate a list of `(hash, len)` pieces under `self.bases`.
    pub fn concat(self, parts: List<(hash: HashVal, len: Int)>) -> HashVal = todo

    /// O(1). Number of hashed elements.
    pub @get fn len(self) -> Int = todo

    /// O(1). True when no elements are hashed.
    pub @get fn is_empty(self) -> Bool = todo

    /// Hash of the entire sequence. O(1).
    pub @get fn all(self) -> HashVal = todo

    @allow(unproven_list_index)
    fn raw_slice(self, l: Int, r: Int) -> HashVal = todo

    @allow(unproven_list_index)
    fn eq_sub(self, a: Int, b: Int, len: Int) -> Bool = todo

    /// O(1). Hash of the half-open 0-based slice `[l, r)`; `HashVal(0, 0)` when `l == r`. Stops on
    /// `l > r` or an end outside the sequence. O(1).
    pub fn slice(self, l: Int, r: Int) -> HashVal = todo

    /// Hash of a range (`Range<Int>` such as `l..r`, `l..=r`, `..k`, or `Range<D>` on `[D]T`). O(1).
    pub fn get<R>(self, r: R) -> HashVal = todo

    /// Concatenate hashes `a` and `b` where `b` has length `b_len`, in O(1).
    pub fn combine(self, a: HashVal, b: HashVal, b_len: Int) -> HashVal = todo

    /// Longest common prefix of suffixes starting at `a` and `b`. O(log n).
    pub fn lcp(self, a: Int, b: Int) -> Int = todo

    /// Longest common suffix of prefixes `s[..a]` and `s[..b]`. O(log n).
    pub fn lcs(self, a: Int, b: Int) -> Int = todo

    /// O(log n). Longest common prefix of `self[a..]` and `other[b..]`. Requires `self.bases == other.bases`.
    /// O(log min(|self|, |other|)).
    pub fn lcp_with(self, a: Int, other: RollingHash, b: Int) -> Int = todo

/// O(R * C) build, O(1) rectangle hash. Double-mod 2D polynomial prefix hash over a grid with row domain `R` and column domain `C`.
/// Built in O(R * C); `rect(rows, cols)` is O(1) and returns a `HashVal`.
pub struct Hash2D<R, C>(bases: (Int, Int)):
    stride: Int
    pre1: List<Int>
    pre2: List<Int>
    pow_r1: List<Int>
    pow_c1: List<Int>
    pow_r2: List<Int>
    pow_c2: List<Int>

    @allow(unproven_list_index)
    /// Build a 2D polynomial rolling hash over grid `(R, C)`. O(|R| * |C|).
    pub fn new(code: (R, C) -> Int, bases: ?(Int, Int) = none) -> Hash2D<R, C> = todo

    /// Hash of the axis-aligned rectangle `rows` x `cols`. O(1).
    @allow(unproven_list_index)
    pub fn rect(self, rows: Range<R>, cols: Range<C>) -> HashVal = todo

/// O(1) value. Single-modulus `2^61 - 1` (Mersenne prime) polynomial hash value (`Copy + Eq + Ord + Hash`).
pub struct Hash61(v: U64)

/// O(n) build, O(1) query. Single-modulus `2^61 - 1` polynomial rolling hash with randomized base.
pub struct RollingHash61:
    base: U64
    pre: List<U64>
    pow: List<U64>

    /// O(n). Build a mod `2^61 - 1` rolling hash over `seq` with random base unless `base:` is given.
    pub fn new<S: Seq>(seq: S, base: ?U64 = none) -> RollingHash61 = todo

    /// O(1). Number of hashed elements.
    pub @get fn len(self) -> Int = todo

    /// O(1). True when no elements are hashed.
    pub @get fn is_empty(self) -> Bool = todo

    /// O(1). Hash of the entire sequence.
    pub @get fn all(self) -> Hash61 = todo

    /// O(1). Hash of range `r` (`Range<Int>` or `Range<D>`).
    pub fn get<R>(self, r: R) -> Hash61 = todo

    /// O(1). Concatenate hashes `a` and `b` where `b` has length `b_len`.
    pub fn combine(self, a: Hash61, b: Hash61, b_len: Int) -> Hash61 = todo

    /// O(1) amortized. Append one element `x`.
    pub fn push<T: Hashable>(mut self, x: T) = todo

    /// O(1). Pop the last element.
    pub fn pop(mut self) = todo

    /// O(log n). Longest common prefix of suffixes starting at `a` and `b`.
    pub fn lcp(self, a: Int, b: Int) -> Int = todo

    /// O(log n). Longest common suffix of prefixes ending at `a` and `b`.
    pub fn lcs(self, a: Int, b: Int) -> Int = todo

    /// O(log min(|self|, |other|)). Longest common prefix of `self[a..]` and `other[b..]`.
    pub fn lcp_with(self, a: Int, other: RollingHash61, b: Int) -> Int = todo

/// O(n) build, O(log n) point update and range hash. Mutable double-mod rolling hash over domain `D`.
/// Supports `h[i] = x` via W8 `IndexSet`.
pub struct DynamicHash<D>:
    bases: (Int, Int)

    /// O(n). Build a mutable rolling hash over `seq` indexed by domain `D`.
    pub fn new<S: Indexed>(seq: S, bases: ?(Int, Int) = none) -> DynamicHash<D> = todo

    /// O(1). Number of hashed positions (`D.K`).
    pub @get fn len(self) -> Int = todo

    /// O(1). True when `D` is empty.
    pub @get fn is_empty(self) -> Bool = todo

    /// O(1). Hash of the entire sequence `..`.
    pub @get fn all(self) -> HashVal = todo

    /// O(log n). Set position `i` to `x` (also powers `h[i] = x` via W8 `IndexSet`).
    pub fn set<T: Hashable>(mut self, i: D, x: T) = todo

    /// O(log n). Hash of the range `r` (accepts cut ranges `j..i` via decision 124).
    pub fn get(self, r: Range<D>) -> HashVal = todo

    /// O(log n). Hash of the reversed slice over range `r` (accepts cut ranges `j..i`).
    pub fn rev_get(self, r: Range<D>) -> HashVal = todo

    /// O(log n). True when the slice over range `r` is a palindrome.
    pub fn is_palindrome(self, r: Range<D>) -> Bool = todo

// ---- alphabet-domain automata

/// O(n * |A|) space, O(1) amortized push. Online suffix automaton over alphabet domain `A`.
/// O(n * |A|) flat memory, O(n) amortized build, O(states) `topo_order` and `occurrences`.
pub struct SuffixAutomaton<A>:
    alpha: 0..=2_000_000_000
    next: List<-1..=2_000_000_000>
    links: List<-1..=2_000_000_000>
    max_len: List<0..=2_000_000_000>
    first_end: List<-1..=2_000_000_000>
    prefix: List<Bool>
    last: 0..=2_000_000_000

    /// Create an empty suffix automaton over alphabet `A`. O(1).
    pub fn new() -> SuffixAutomaton<A> = todo

    /// Build the suffix automaton of `s`. O(|s| * |A|) memory, O(|s|) amortized time.
    pub fn of<S: Seq>(s: S) -> SuffixAutomaton<A> = todo

    @allow(unproven_list_index)
    fn add_empty_state(mut self, length: Int, is_prefix: Bool) -> Int = todo

    @allow(unproven_list_index)
    fn clone_state(mut self, src: Int, length: Int) -> Int = todo

    /// Append one letter `a`. O(1) amortized.
    @allow(unproven_list_index)
    pub fn push(mut self, a: A) = todo

    /// Root state (empty string): always `0`. O(1).
    pub @get fn root(self) -> Int = todo

    /// Number of states. O(1).
    pub @get fn len(self) -> Int = todo

    /// Transition from `state` on `a`, or `none`. O(1).
    @allow(unproven_list_index)
    pub fn step(self, state: Int, a: A) -> ?Int = todo

    /// Suffix link of `state`, or `none` at the root. O(1).
    @allow(unproven_list_index)
    pub fn link(self, state: Int) -> ?Int = todo

    /// Length of the longest substring in `state`. O(1).
    @allow(unproven_list_index)
    pub fn longest(self, state: Int) -> Int = todo

    /// Length of the shortest substring in `state` (`0` at the root). O(1).
    @allow(unproven_list_index)
    pub fn shortest(
        self,
        state: Int,
    ) -> Int = todo

    /// 0-based right endpoint of the first occurrence of `state` (`-1` at the root). O(1).
    @allow(unproven_list_index)
    pub fn first_pos(self, state: Int) -> Int = todo

    /// State of `word`, or `none` if `word` is not a substring. O(|word|).
    @allow(unproven_list_index)
    pub fn find<S: Seq>(self, word: S) -> ?Int = todo

    /// True when `word` is a substring of the indexed string. O(|word|).
    pub fn contains<S: Seq>(self, word: S) -> Bool = todo

    /// Number of distinct non-empty substrings. O(states).
    pub fn distinct(self) -> Int = todo

    /// All states sorted by non-decreasing `longest(state)` via counting sort. O(states).
    @allow(unproven_list_index)
    pub fn topo_order(self) -> List<Int> = todo

    /// O(states). Number of end positions (occurrence count) of each state's substrings in the indexed string.
    @allow(unproven_list_index)
    pub fn occ_counts(self) -> List<Int> = todo

    /// O(|other|). Longest common substring of the indexed string and `other`: returns `(len, end)` where
    /// `other[end - len..end]` is a longest common substring. O(|other|).
    @allow(unproven_list_index)
    pub fn lcs<S: Seq>(self, other: S) -> (len: Int, end: Int) = todo

    /// O(1). Reset `last` to `root` to start appending another word into a generalized SAM.
    pub fn reset(mut self) = todo

    /// O(|word|) amortized. Insert `word` into a generalized SAM (`reset()` then `push` each letter).
    pub fn add_word<S: Seq>(mut self, word: S) = todo

    /// O(states). Yield all states in topological order (non-decreasing `longest`).
    pub fn states(self) -> yields Int = todo

    /// O(states). Children lists of the suffix-link (endpos) tree, rooted at `0`.
    pub fn endpos_tree(self) -> List<List<Int>> = todo

    /// O(states * |A|). The `k`-th lexicographically smallest non-empty substring (0-based), or `none`.
    /// Set `distinct: false` to count substrings with multiplicity.
    pub fn kth_substring(self, k: Int, distinct: Bool = true) -> ?List<A> = todo

/// O(sum |w|). Length of the longest common substring shared by all sequences in `words`.
pub fn lcs_all<A, S: Seq>(words: List<S>) -> Int = todo

/// O(n * |A|) space, O(1) amortized push. Online palindromic tree (eertree) over alphabet domain `A`, with `diff` and `series_link`
/// for O(n log n) palindromic partition DP.
/// State `0` is the odd root (`pal_len == -1`); state `1` is the even root (`pal_len == 0`).
pub struct PalindromicTree<A>:
    alpha: 0..=2_000_000_000
    next: List<-1..=2_000_000_000>
    slink: List<0..=2_000_000_000>
    plen: List<-1..=2_000_000_000>
    diff_val: List<0..=2_000_000_000>
    ser_link: List<0..=2_000_000_000>
    cnt: List<0..=2_000_000_000>
    chars: List<A>
    last: 0..=2_000_000_000

    /// Create an empty palindromic tree (eertree) over alphabet `A`. O(1).
    pub fn new() -> PalindromicTree<A> = todo

    /// Build the palindromic tree of `s`. O(|s| * |A|) flat memory, O(|s|) amortized time.
    pub fn of<S: Seq>(s: S) -> PalindromicTree<A> = todo

    /// O(1) amortized. Append one letter `a` and return the state of the longest palindromic suffix.
    /// O(1) amortized.
    @allow(unproven_list_index)
    pub fn push(mut self, a: A) -> Int = todo

    /// Total number of states, including the odd (`0`) and even (`1`) roots. O(1).
    pub @get fn len(self) -> Int = todo

    /// Number of distinct non-empty palindromic substrings. O(1).
    pub @get fn distinct(self) -> Int = todo

    /// Length of the palindrome represented by `state` (`-1` for odd root, `0` for even root). O(1).
    @allow(unproven_list_index)
    pub fn pal_len(self, state: Int) -> Int = todo

    /// Longest proper palindromic suffix state of `state`. O(1).
    @allow(unproven_list_index)
    pub fn link(self, state: Int) -> Int = todo

    /// Transition from `state` wrapping with `a` on both ends, or `none`. O(1).
    @allow(unproven_list_index)
    pub fn step(self, state: Int, a: A) -> ?Int = todo

    /// Period `pal_len(state) - pal_len(link(state))`. O(1).
    @allow(unproven_list_index)
    pub fn diff(self, state: Int) -> Int = todo

    /// Nearest suffix-link ancestor with a different `diff`, for O(log n) series DP. O(1).
    @allow(unproven_list_index)
    pub fn series_link(self, state: Int) -> Int = todo

    /// O(states). Occurrence count of each palindrome state in the pushed string.
    @allow(unproven_list_index)
    pub fn occ_counts(self) -> List<Int> = todo

    /// O(1) amortized. Prepend one letter `a` (double-ended eertree) and return the longest palindromic prefix state.
    pub fn push_front(mut self, a: A) -> Int = todo

    /// O(1) amortized. Roll back the most recent `push` (for DFS on trees).
    pub fn pop(mut self) = todo

    /// O(1). State of the longest palindromic suffix after the most recent `push`.
    pub @get fn last_state(self) -> Int = todo

    /// O(|word|). True when `word` is a palindromic substring currently in the tree.
    pub fn contains<S: Seq>(self, word: S) -> Bool = todo

    /// O(n log n). `dp[i]` is the minimum number of palindromes partitioning the first `i` pushed letters (`0..=n`).
    pub fn min_palindrome_partition(self) -> List<Int> = todo

/// Dynamic trie over alphabet domain `A`. O(|word|) per operation; flat `|A|`-stride table.
pub struct Trie<A>:
    alpha: 0..=2_000_000_000
    next: List<-1..=2_000_000_000>
    ends: List<0..=2_000_000_000>
    pass: List<0..=2_000_000_000>

    /// Create an empty prefix trie over alphabet `A`. O(1).
    pub fn new() -> Trie<A> = todo

    /// Root node (empty word): always `0`. O(1).
    pub @get fn root(self) -> Int = todo

    /// Total number of nodes, root included. O(1).
    pub @get fn len(self) -> Int = todo

    /// O(1). True when no words are stored in the trie.
    pub @get fn is_empty(self) -> Bool = todo

    /// Child of `node` along edge `a`, or `none`. O(1).
    @allow(unproven_list_index)
    pub fn step(self, node: Int, a: A) -> ?Int = todo

    /// Node of `word`, or `none` if no added word starts with it. O(|word|).
    @allow(unproven_list_index)
    pub fn find<S: Seq>(self, word: S) -> ?Int = todo

    /// Insert one copy of `word` and return its terminal node. O(|word|).
    @allow(unproven_list_index)
    pub fn add<S: Seq>(mut self, word: S) -> Int = todo

    /// Remove one copy of `word`. Returns `false` (no change) if absent. O(|word|).
    @allow(unproven_list_index)
    pub fn remove<S: Seq>(mut self, word: S) -> Bool = todo

    /// Copies of `word` currently in the trie. O(|word|).
    @allow(unproven_list_index)
    pub fn count<S: Seq>(self, word: S) -> Int = todo

    /// Added words starting with `prefix`. O(|prefix|).
    @allow(unproven_list_index)
    pub fn count_prefix<S: Seq>(self, prefix: S) -> Int = todo

    /// Words ending at `node`. O(1).
    @allow(unproven_list_index)
    pub fn ends_at(self, node: Int) -> Int = todo

    /// Words passing through or ending at `node`. O(1).
    @allow(unproven_list_index)
    pub fn pass_at(self, node: Int) -> Int = todo

    /// O(|word|). True when at least one copy of `word` is in the trie.
    pub fn contains<S: Seq>(self, word: S) -> Bool = todo

    /// O(1). Remove all words and reset to an empty root node.
    pub fn clear(mut self) = todo

    /// O(depth * |A|). The `k`-th lexicographically smallest word in the trie (0-based, with multiplicity), or `none`.
    pub fn kth(self, k: Int) -> ?List<A> = todo

    /// O(|word| * |A|). Number of words in the trie strictly smaller than `word`.
    pub fn count_less<S: Seq>(self, word: S) -> Int = todo

/// O(total * |A|) build, O(1) step. Aho-Corasick multi-pattern automaton over alphabet domain `A`.
/// Built in O(sum |p| * |A|) flat memory and time; `step`, `link`, and `hit` are O(1).
pub struct AhoCorasick<A>:
    alpha: 0..=2_000_000_000
    goto: List<-1..=2_000_000_000>
    fail: List<0..=2_000_000_000>
    ends: List<List<0..=2_000_000_000>>
    exit: List<-1..=2_000_000_000>
    order: List<0..=2_000_000_000>
    lens: List<0..=2_000_000_000>

    @allow(unproven_list_index)
    /// Build an Aho-Corasick automaton over `patterns`. O(total pattern length).
    pub fn new<S: Seq>(patterns: List<S>) -> AhoCorasick<A> = todo

    /// Root state (before reading any letter): always `0`. O(1).
    pub @get fn root(self) -> Int = todo

    /// Number of states. O(1).
    pub @get fn len(self) -> Int = todo

    /// Total transition from `state` on `a`. O(1).
    @allow(unproven_list_index)
    pub fn step(self, state: Int, a: A) -> Int = todo

    /// Failure (suffix) link of `state` (root links to `0`). O(1).
    @allow(unproven_list_index)
    pub fn link(self, state: Int) -> Int = todo

    /// True when any pattern ends at `state` or along its exit links. O(1).
    @allow(unproven_list_index)
    pub fn hit(self, state: Int) -> Bool = todo

    /// All pattern indices ending at `state`, longest first. O(output).
    @allow(unproven_list_index)
    pub fn words_at(self, state: Int) -> List<Int> = todo

    /// O(|text| + matches). Yield every occurrence `(at, pattern)` in `text`, ordered by end position.
    /// O(|text| + output).
    @allow(unproven_list_index)
    pub fn find_all<S: Seq>(self, text: S) -> yields (at: Int, pattern: Int) = todo

    /// Occurrence count of every pattern in `text` (overlaps included). O(|text| + states).
    @allow(unproven_list_index)
    pub fn counts<S: Seq>(self, text: S) -> List<Int> = todo

    /// O(states). BFS order of states for bottom-up DP on the failure tree.
    pub fn fail_order(self) -> List<Int> = todo

    /// O(1). Nearest strict ancestor in the failure tree where a pattern ends, or `none`.
    pub fn exit_link(self, state: Int) -> ?Int = todo

    /// O(1). Terminal state of `patterns[pattern]`.
    pub fn pattern_state(self, pattern: Int) -> Int = todo

/// O(|pat| * |A| * log n) amortized insert/delete, O(|text| * log n) query.
/// Online multi-pattern Aho-Corasick via log-decomposed static buckets.
pub struct DynamicAhoCorasick<A>:
    alpha: 0..=2_000_000_000

    /// O(1). Create an empty online Aho-Corasick pattern set over alphabet `A`.
    pub fn new() -> DynamicAhoCorasick<A> = todo

    /// O(1). Number of active patterns currently stored (with multiplicity).
    pub @get fn len(self) -> Int = todo

    /// O(1). True when no active patterns are stored.
    pub @get fn is_empty(self) -> Bool = todo

    /// O(|pattern| * |A| * log n) amortized. Insert one copy of `pattern`.
    pub fn add<S: Seq>(mut self, pattern: S) = todo

    /// O(|pattern| * |A| * log n) amortized. Remove one copy of `pattern`.
    pub fn remove<S: Seq>(mut self, pattern: S) = todo

    /// O(|text| * log n). Total occurrences of all active patterns in `text` (overlaps included).
    pub fn count_matches<S: Seq>(self, text: S) -> Int = todo

// ---- tests

fn random_word(n: Int, letters: Int) -> List<Char> = todo

fn common_prefix<T>(s: List<T>, a: Int, b: Int) -> Int = todo

fn suffix_less<T>(s: List<T>, a: Int, b: Int) -> Bool = todo

@test
fn z_and_prefix_function_match_their_definitions() = todo

@test
fn find_all_matches_a_scan() = todo

@test
fn suffix_array_and_lcp_match_sorting_the_suffixes() = todo

@test
fn rolling_hash_over_str_list_and_domain_arrays() = todo

@test
fn repeat_writes_a_string_over() = todo

@test
fn starts_ends_and_find_on_text() = todo

@test
fn lower_and_upper_change_only_letters() = todo

@test
fn digit_is_the_value_of_a_decimal_character() = todo

@test
fn count_counts_one_character() = todo

@test
fn rfind_gives_the_last_place() = todo

@test
fn borders_prefix_matches_and_overlap_match_brute() = todo

@test
fn lyndon_factorization_and_min_rotation_match_definitions() = todo

@test
fn manacher_and_is_palindrome_match_brute() = todo

@test
fn suffix_array_struct_queries_match_brute() = todo

@test
fn rolling_hash_lcp_lcs_and_hash2d_match_brute() = todo

@test
fn suffix_automaton_matches_brute_and_linear_occurrences() = todo

@test
fn palindromic_tree_distinct_occurrences_and_series_dp() = todo

@test
fn trie_add_remove_and_prefix_counts() = todo

@test
fn aho_corasick_matches_counts_and_avoidance_dp() = todo

@test
fn indexed_views_and_common_prefix_suffix() = todo

@test
fn lyndon_runs_matches_brute() = todo

@test
fn sequence_generality_parity_across_list_array_and_views() = todo
