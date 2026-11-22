# `std.strings`: complete design

> **0.3.0 status.** This chapter is the family's design survey and API.
> Where it disagrees with the core chapters or the aligned rounds, they
> win: `core/03-numbers-layout.md` (four number types, no `I32`/`U32`,
> `Mod<P>` not `Mint`), `core/05-ownership.md` (constructors own their
> results), `core/07-sequences.md` (capabilities, views keep keys),
> `lib/ops-ranges.md` (`Op`/`Monoid`/laws, `each(...)`, no
> `RangeFenwick`, `MinHeap`/`MaxHeap` as `Heap(by:)`) and
> `lib/sequence-patterns.md`. The stubs in `std/` are the signatures of
> record. Reconciliation of this text is tracked in `delta/`.


This document designs the `std.strings` area of the CP Lang standard library
top down: the contest patterns it must cover, one coherent API with costs and
laws, how it uses domains and index spaces, what stays user code, and five real
contest problems written against it.

Facts checked directly in this repository (`stdlib/strings.cp`,
`examples/local-lib/str/`, `proto/ports/ecnerwala/ucup5s05f_orig.cpp`,
`proto/ports/scout_batch_10/`, `research/friction-backlog-batch-10.md`) are
verified; external library inventories (KACTL, AtCoder Library, CP-Algorithms)
are standard public references.

## 1. Survey of contest patterns

String problems across Codeforces Div. 1, AtCoder Grand/WTF, ICPC World Finals,
and IOI cluster into six families.

**Pattern matching, borders, and periodicity.** Single-pattern search, prefix
function (KMP), Z-function, enumerating all borders of a string, shortest
period, longest suffix-prefix overlap between two strings, and matching lengths
of a pattern across a text without inventing a sentinel outside the alphabet.
KACTL ships `KMP` (`pi`, `match`) and `Zfunc`; AtCoder Library ships
`z_algorithm`. In our corpus, `ctsc06p1` sums `m^b` over all borders,
`noip20p2` uses `z_function` to count `(AB)^k C` splits, and `btoi19p5` wants
the longest prefix of a pattern ending at each text position (`T5` in
`research/friction-backlog-batch-10.md`).

**Lyndon words and cyclic rotations.** Lexicographically smallest cyclic shift
and Duval's Lyndon factorization. KACTL ships `MinRotation` (Booth/Duval,
O(n)); CP-Algorithms pairs it with `lyndon_factorization` (Duval, O(n)), from
which `min_rotation` is five lines. Both are short, notorious for off-by-one
bugs under contest pressure, and work on any `Ord` sequence.

**Polynomial rolling hashing.** O(1) substring equality, concatenation of
hashes, O(log n) LCP/LCS via binary search, lexicographic substring comparison,
and 2D grid rectangle hashing. KACTL ships `Hashing` (arithmetic mod `2^64 - 1`
with fixed or randomized base) and `Hashing-codeforces` (double modulus against
anti-hash hacks). Six of ten submissions in `scout_batch_10` hand-rolled their
own hasher (`noi16p1`, `noi17p2`, `year2018p7`, `noip20p2`, `btoi19p5`,
`nccc10s3`). `stdlib/strings.cp` already provides `HashVal` and `RollingHash`
(double-mod `10^9 + 7` and `10^9 + 9`, random per-instance bases, composable
into 2D via `HashVal.hash_token`), and `examples/local-lib/str/hash2d.cp`
provides `Hash2D<R, C>`. What is missing from `RollingHash` is `lcp`, `lcs`,
and `lcp_with` (`noi16p1`), plus promoting `Hash2D<R, C>` with double-mod
`HashVal` so single-mod collisions are impossible on Codeforces.

**Suffix structures.** Suffix array (SA-IS, O(n) for bounded alphabets and
O(n log n) for general `Ord`), Kasai LCP array (O(n)), O(1) arbitrary-pair LCP
queries via RMQ, pattern range search in O(|pat| log n), O(1) substring
comparison, and the suffix automaton (SAM, online O(n) amortized) for distinct
substrings, endpos counts, and longest common substring. AtCoder Library ships
`suffix_array` (SA-IS) and `lcp_array` (Kasai). KACTL ships `SuffixArray` and
`SuffixTree`. In `proto/ports/ecnerwala/ucup5s05f_orig.cpp`, ecnerwala's
`src/seq/suffix_array.hpp` layers `SuffixArrayBase` (`sa`, `rank`),
`SuffixArrayLCPBase` (`lcp`), `SuffixArrayRMQBase` (`get_lcp(a, b)`,
`get_split(l, r)`), and `PrefixArrayRMQ` (`get_lcs(a, b)`). In our corpus,
`saspeed` tests O(n) SA-IS speed at `n = 10^6`, `noi15p5` joins adjacent
suffixes by decreasing LCP, `nccc10s3` binary-searches the suffix array for
pattern occurrences (`T4`), and `ucup5s05f` walks the suffix tree over `sa` and
`lcp`. `examples/local-lib/str/suffix_automaton.cp` already has a clean
`SuffixAutomaton<A>`, though its `occurrences()` uses O(n log n) sort instead
of O(n) counting sort.

**Palindromes.** Manacher's algorithm for all odd/even palindrome radii in
O(n), O(1) substring palindrome queries, total palindrome count, longest
palindromic substring, and the palindromic tree (Eertree) for distinct
palindromes, per-palindrome occurrence counts, and O(n log n) palindrome
partition DP via series links. KACTL ships `Manacher`; `examples/local-lib/str/
palindromes.cp` has Manacher's radii. `plan/STDLIB.md` already schedules
`manacher` and `is_palindrome`.

**Tries and multi-pattern matching.** Static/dynamic prefix tries over an
alphabet domain (`Trie<A>`) for dictionary lookup, prefix counting, and binary
XOR maximization (`ioi08p1`, `sac22cc3p5`), and Aho-Corasick (`AhoCorasick<A>`)
for simultaneous multi-pattern matching, occurrence counting, and DP over
strings avoiding or containing forbidden words. KACTL ships `AhoCorasick`;
`examples/local-lib/str/trie.cp` and `examples/local-lib/str/aho_corasick.cp`
already demonstrate the alphabet-domain pattern.

## 2. Design principles and index spaces

**Flat 0-based positions over any `Indexed` / `Seq` sequence.** Per decision
1.6, `Str` is a flat `0..len`-indexed sequence of 1-byte ASCII `Char` values.
All sequence-level algorithms (`z_function`, `prefix_function`, `find_all`,
`common_prefix_len`, `common_suffix_len`, `period`, `borders`,
`prefix_matches`, `overlap`, `lyndon_factorization`, `min_rotation`,
`lyndon_runs`, `is_palindrome`, `manacher`, `suffix_array`, `lcp_array`,
`SuffixArray`) are generic over `Indexed` (`Str`, `StrView`, `List<T>`,
`ListView<T>`, and `[D]T` domain arrays) and return 0-based `Int` positions
and lengths; all alphabet-domain automata (`SuffixAutomaton`, `PalindromicTree`,
`Trie`, `AhoCorasick`) accept any `Seq` of alphabet elements.

**Alphabet domains on automata.** `Trie<A>`, `AhoCorasick<A>`,
`SuffixAutomaton<A>`, and `PalindromicTree<A>` take an alphabet domain `A`
(such as `domain Letter = 'a'..='z'`, `domain Bit = 0..2`, or
`domain Digit = '0'..='9'`). Every state stores its transitions as `[A]?Int` or
`[A]Int`. This beats hardcoding 26 lowercase letters on every axis: binary
alphabets use 2 slots instead of 26, uppercase/digit/general alphabets work
without `c - 'a'` arithmetic, and invalid characters are caught when entering
the domain.

**Domain-aware hashing.** `RollingHash` accepts `Str`, `List<T>`, `[D]T`, or
`[D]HashVal`, and `.get(r)` accepts `Range<Int>` or `Range<D>`. `Hash2D<R, C>`
is parameterized by the grid's row and column domains `R` and `C`, taking
`Range<R>` and `Range<C>` in `.rect(rows, cols)`, so row/column swaps are type
errors.

**Layered cost for suffix arrays.** Computing only `sa` and `lcp` is O(n) via
the free functions `suffix_array(s)` and `lcp_array(s, sa)`. When a solution
also wants `rank`, O(1) `lcp_between(i, j)`, O(|pat| log n) `matches(pat)`,
O(log n) `lcp_range(i, len)`, or O(1) `compare`, it constructs `SuffixArray(s)`
at documented O(n log n) build cost. Nothing is built secretly behind
`suffix_array(s)`.

**Zero-cost safety and guessable names.** Every algorithm uses its textbook
name (`z_function`, `prefix_function`, `manacher`, `suffix_array`,
`lcp_array`, `lyndon_factorization`, `min_rotation`, `AhoCorasick`,
`SuffixAutomaton`, `PalindromicTree`, `Trie`, `RollingHash`, `Hash2D`). Options
are named arguments (`bases:`). Container-like structures follow `plan/STDLIB.md`
§2 (`len`, `root`, `step`, `find`, `contains`, `add`, `remove`, `count`).

## 3. Coherent API, costs, and laws

### 3.1 `Str` and `Char` helpers

```cp
fn repeat(self: Str, n: Int) -> Str                 // O(n * self.len)
fn starts_with(self: Str, p: Str) -> Bool           // O(p.len)
fn ends_with(self: Str, p: Str) -> Bool             // O(p.len)
fn find<P>(self: Str, p: P) -> ?Int                 // O(self.len + p.len)
fn rfind<P>(self: Str, p: P) -> ?Int                // O(self.len + p.len)
fn count(self: Str, c: Char) -> Int                 // O(self.len)
@get fn lower(self: Char) -> Char                   // O(1)
@get fn upper(self: Char) -> Char                   // O(1)
@get fn digit(self: Char) -> Int                    // O(1); traps outside '0'..'9'
fn is_palindrome<S: Indexed>(s: S) -> Bool          // O(n)
fn common_prefix_len<A: Indexed, B: Indexed>(self: A, other: B) -> Int  // O(min)
fn common_suffix_len<A: Indexed, B: Indexed>(self: A, other: B) -> Int  // O(min)
```

Laws:
- `s.repeat(0) == ""` and `s.repeat(a + b) == s.repeat(a) + s.repeat(b)`.
- `s.find(p) == Some(i)` implies `i` is the minimum index with `s[i..i + p.len] == p`.
- `is_palindrome(s)` iff `s[i] == s[n - 1 - i]` for all `0 <= i < n`.

### 3.2 Pattern matching, borders, periods, and Lyndon words

```cp
fn z_function<S: Indexed>(s: S) -> List<Int>                    // O(n)
fn prefix_function<S: Indexed>(s: S) -> List<Int>               // O(n)
fn find_all<S: Indexed, P: Indexed>(self: S, pat: P) -> yields Int // O(|self| + |pat|)
fn period<S: Indexed>(s: S) -> Int                              // O(n)
fn borders<S: Indexed>(s: S) -> yields Int                      // O(n); longest first
fn prefix_matches<S: Indexed, P: Indexed>(self: S, pat: P) -> List<Int>
    // O(|self| + |pat|); entry i is the longest prefix of pat ending at self[i]
fn overlap<A: Indexed, B: Indexed>(a: A, b: B) -> Int
    // O(|a| + |b|); longest suffix of a that is a prefix of b
fn lyndon_factorization<S: Indexed>(s: S) -> List<Int>
    // O(n) Duval; start indices of the unique non-increasing Lyndon factors
fn min_rotation<S: Indexed>(s: S) -> Int
    // O(n) Duval/Booth; start index of the lexicographically least cyclic shift
fn lyndon_runs<S: Indexed>(s: S) -> List<(l: Int, r: Int, period: Int)>
    // O(n log n) Bannai runs theorem; all maximal periodic repetitions
```

Laws:
- `z_function(s)[i] == lcp(s, s[i..])` for every `i in 0..n`.
- `prefix_function(s)[i]` is the maximum `k <= i` with `s[..k] == s[i + 1 - k..=i]`.
- `borders(s)` yields every `k in 1..n` with `s[..k] == s[n - k..]`, strictly
  decreasing.
- `period(s)` is `0` when `s.is_empty`, else the minimum `d in 1..=n` with
  `s[i] == s[i + d]` for all `0 <= i < n - d`.
- `prefix_matches(text, pat)[i]` is the maximum `k <= min(i + 1, pat.len)` with
  `pat[..k] == text[i + 1 - k..=i]`, and `i + 1 - pat.len` is yielded by
  `find_all(text, pat)` iff `prefix_matches(text, pat)[i] == pat.len`.
- `overlap(a, b)` is the maximum `k <= min(a.len, b.len)` with
  `a[a.len - k..] == b[..k]`.
- For `starts = lyndon_factorization(s)`, every factor `w_j` is strictly smaller
  than all its proper non-empty suffixes, and `w_0 >= w_1 >= ...`.
- `min_rotation(s)` achieves the minimum cyclic shift `s[r..] + s[..r]` over
  all `r in 0..n` (and `0` when `n == 0`).

### 3.3 Palindromes

```cp
struct Palindromes(odd: List<Int>, even: List<Int>):
    fn is_palindrome<R>(self, r: R) -> Bool          // O(1); Range<Int> or Range<D>
    @get fn count(self) -> Int                       // O(n); total palindromic substrings
    @get fn longest(self) -> (l: Int, r: Int)        // O(n); longest palindromic range

fn manacher<S: Indexed>(s: S) -> Palindromes         // O(n)

struct PalindromicTree<A>:
    fn new() -> PalindromicTree<A>                   // O(|A|)
    fn of<S: Seq>(s: S) -> PalindromicTree<A>        // O(n * |A|) mem, O(n) time
    fn push(mut self, a: A) -> Int                   // O(1) amortized; returns last state
    @get fn len(self) -> Int                         // O(1); states incl. 2 roots
    @get fn distinct(self) -> Int                    // O(1); len - 2
    fn pal_len(self, state: Int) -> Int              // O(1)
    fn link(self, state: Int) -> Int                 // O(1); longest proper pal suffix
    fn step(self, state: Int, a: A) -> ?Int          // O(1); a + P + a
    fn diff(self, state: Int) -> Int                 // O(1); pal_len - pal_len(link)
    fn series_link(self, state: Int) -> Int          // O(1); nearest ancestor with other diff
    fn occurrences(self) -> List<Int>                // O(n); total occurrences per state
```

Laws:
- For `p = manacher(s)`: `p.odd[i]` is the maximum `k` with `s[i - k + 1..i + k]`
  palindromic; `p.even[i]` is the maximum `k` with `s[i - k..i + k]` palindromic.
- `p.is_palindrome(l..r) == is_palindrome(s[l..r])` for all `0 <= l <= r <= n`.
- `p.count == sum(p.odd) + sum(p.even)`, and equals
  `sum(pt.occurrences()[2..])` on `pt = PalindromicTree<A>.of(s)`.
- `pt.distinct` equals the number of distinct non-empty palindromic substrings
  of `s`, and `pt.distinct <= n`.
- Walking `state = pt.series_link(state)` from any state to root 0 takes at
  most `O(log n)` steps.

### 3.4 Rolling hashing

```cp
struct HashVal(h1: Int, h2: Int):
    @get fn hash_token(self) -> Int                  // O(1)

trait Hashable:
    @get fn hash_token(self) -> Int = Int(self)

struct RollingHash:
    fn new<S>(seq: S, bases: ?(Int, Int) = None) -> RollingHash       // O(n)
    fn with_bases<S>(seq: S, bases: (Int, Int)) -> RollingHash        // O(n)
    @get fn len(self) -> Int                                          // O(1)
    @get fn all(self) -> HashVal                                      // O(1)
    fn slice(self, l: Int, r: Int) -> HashVal                         // O(1)
    fn get<R>(self, r: R) -> HashVal                                  // O(1)
    fn combine(self, a: HashVal, b: HashVal, b_len: Int) -> HashVal   // O(1)
    fn lcp(self, a: Int, b: Int) -> Int                               // O(log n)
    fn lcs(self, a: Int, b: Int) -> Int                               // O(log n)
    fn lcp_with(self, a: Int, other: RollingHash, b: Int) -> Int      // O(log n)

struct Hash2D<R, C>:
    fn new(code: (R, C) -> Int, bases: ?(Int, Int) = None) -> Hash2D<R, C>  // O(R * C)
    fn rect(self, rows: Range<R>, cols: Range<C>) -> HashVal                // O(1)
```

Laws:
- Equal slices always produce equal `HashVal`s; distinct slices collide with
  probability at most `len / (10^9 + 7) * len / (10^9 + 9) < 10^-18 * len^2`.
- `h.combine(h.slice(l, m), h.slice(m, r), r - m) == h.slice(l, r)` for all
  `0 <= l <= m <= r <= h.len`.
- `h.lcp(a, b)` equals the exact LCP of `s[a..]` and `s[b..]` whp; `h.lcs(a, b)`
  equals the exact LCS of `s[..a]` and `s[..b]` whp.
- `Hash2D.rect` returns equal `HashVal`s for identical sub-rectangles of the
  same shape; sharing `bases:` makes rectangles comparable across two grids.

### 3.5 Suffix structures

```cp
fn suffix_array<S: Indexed>(s: S) -> List<Int>          // O(n) if <=256 distinct, else O(n log n)
fn lcp_array<S: Indexed>(s: S, sa: List<Int>) -> List<Int>  // O(n) Kasai

struct SuffixArray<T>:
    s: List<T>
    sa: List<Int>
    rank: List<Int>
    lcp: List<Int>

    fn new<S: Indexed>(s: S) -> SuffixArray<T>       // O(n log n) build (SA-IS + Kasai + RMQ)
    @get fn len(self) -> Int                         // O(1)
    fn lcp_between(self, i: Int, j: Int) -> Int      // O(1)
    fn matches<P: Indexed>(self, pat: P) -> Range<Int> // O(|pat| log n); rank range lo..hi
    fn count<P: Indexed>(self, pat: P) -> Int        // O(|pat| log n)
    fn lcp_range(self, i: Int, len: Int) -> Range<Int>
        // O(log n); rank range whose LCP with suffix i is >= len
    fn compare(self, l1: Int, r1: Int, l2: Int, r2: Int) -> Int
        // O(1); -1, 0, or 1 comparing s[l1..r1] vs s[l2..r2]
    @get fn distinct(self) -> Int                    // O(n); distinct non-empty substrings

struct SuffixAutomaton<A>:
    fn new() -> SuffixAutomaton<A>                   // O(|A|)
    fn of<S: Seq>(s: S) -> SuffixAutomaton<A>        // O(n * |A|) mem, O(n) time
    fn push(mut self, a: A)                          // O(1) amortized
    @get fn root(self) -> Int                        // O(1); 0
    @get fn len(self) -> Int                         // O(1); <= max(2n - 1, 1)
    fn step(self, state: Int, a: A) -> ?Int          // O(1)
    fn link(self, state: Int) -> ?Int                // O(1)
    fn longest(self, state: Int) -> Int              // O(1)
    fn shortest(self, state: Int) -> Int             // O(1)
    fn first_pos(self, state: Int) -> Int            // O(1); end index of first occurrence
    fn find<S: Seq>(self, word: S) -> ?Int           // O(|word|)
    fn contains<S: Seq>(self, word: S) -> Bool       // O(|word|)
    @get fn distinct(self) -> Int                    // O(states)
    fn topo_order(self) -> List<Int>                 // O(states); decreasing longest
    fn occurrences(self) -> List<Int>                // O(states); endpos count per state
```

Laws:
- `sa` is a permutation of `0..n` sorting suffixes strictly ascending;
  `rank[sa[j]] == j`.
- `lcp[j] == lcp_between(sa[j], sa[j + 1])` for all `0 <= j < n - 1`.
- `lcp_between(i, i) == n - i` and `lcp_between(i, j) == lcp_between(j, i)`.
- `sa[j]` starts with `pat` iff `j in sa_obj.matches(pat)`.
- `sa_obj.distinct == sam.distinct` for any string `s`.
- `sam.contains(w)` iff `sa_obj.count(w) > 0`; when `sam.find(w) == Some(v)`,
  `sam.occurrences()[v] == sa_obj.count(w)`.

### 3.6 Tries and Aho-Corasick

```cp
struct Trie<A>:
    fn new() -> Trie<A>                              // O(|A|)
    @get fn root(self) -> Int                        // O(1); 0
    @get fn len(self) -> Int                         // O(1); nodes incl. root
    fn step(self, node: Int, a: A) -> ?Int           // O(1)
    fn find<S: Seq>(self, word: S) -> ?Int           // O(|word|)
    fn add<S: Seq>(mut self, word: S) -> Int         // O(|word|); returns terminal node
    fn remove<S: Seq>(mut self, word: S) -> Bool     // O(|word|)
    fn count<S: Seq>(self, word: S) -> Int           // O(|word|)
    fn count_prefix<S: Seq>(self, prefix: S) -> Int  // O(|prefix|)
    fn ends_at(self, node: Int) -> Int               // O(1)
    fn pass_at(self, node: Int) -> Int               // O(1)

struct AhoCorasick<A>:
    fn new<S: Seq>(patterns: List<S>) -> AhoCorasick<A>  // O(sum |p| * |A|)
    @get fn root(self) -> Int                          // O(1); 0
    @get fn len(self) -> Int                           // O(1); states incl. root
    fn step(self, state: Int, a: A) -> Int             // O(1); total DFA transition
    fn link(self, state: Int) -> Int                   // O(1); failure link
    fn hit(self, state: Int) -> Bool                   // O(1); any pattern ends here
    fn words_at(self, state: Int) -> List<Int>         // O(output); longest first
    fn matches<S: Seq>(self, text: S) -> yields (at: Int, pattern: Int)
        // O(|text| + output); ordered by end position
    fn counts<S: Seq>(self, text: S) -> List<Int>      // O(|text| + states)
```

Laws:
- On `Trie<A>`, `add` increments `count(w)` and `count_prefix(p)` for every
  prefix `p` of `w` by 1; `remove` reverses `add` when `count(w) > 0` and is a
  no-op returning `false` otherwise.
- On `AhoCorasick<A>`, `matches(text)` yields `(at, p)` iff
  `text[at..at + patterns[p].len] == patterns[p]`, and `counts(text)[p]` equals
  the number of yielded pairs with `pattern == p`.

## 4. What stays user code

- **Mergeable, persistent, and custom-aggregate tries** (such as `cf2261g`'s
  suffix-AND binary trie merging): every problem attaches different node fields
  and merge semantics; a 15-line `struct Node` + `List<Node>` arena is clearer
  and faster than a bloated generic hook.
- **Explicit suffix trees** (such as `ucup5s05f`'s suffix-tree edge power
  sums): `suffix_array` + `lcp_array` with a 15-line monotonic stack, or the
  suffix-link tree of `SuffixAutomaton<A>`, already gives the suffix tree
  without locking in an edge-label representation.
- **Automaton DP** (avoiding-words DP over `AhoCorasick`, palindrome
  factorization DP over `PalindromicTree`, LCS over `SuffixAutomaton`): the
  automata expose `root`, `len`, `step`, `link`, `hit`, `diff`, `series_link`,
  and `topo_order`, so the DP recurrence itself is 8–15 lines of transparent
  user code.
- **Unicode, regex, and locale**: out of scope for competitive programming
  (`Str` is 1-byte ASCII).

## 5. Five real problems written against the API

### 5.1 CTSC 2006 — Singing Kingdom (`ctsc06p1`)

Expected number of random draws from `1..=m` until `name` appears as a
contiguous block, mod `10^4` (four digits). By the martingale argument, the
expected wait is the sum of `m^b` over every border `b` of `name`, including
`name` itself.

```cp
// Expected wait for a word over 1..=m is sum(m^b) over every border b
// (the whole word plus every proper non-empty border).
use std.strings.{borders}

type Z = Mod<10_000>
let (m, t): (Int, Int) = read()
repeat t:
    let n: Int = read()
    let name: [0..n]Int = read()
    var wait = Z(m) ** n
    for b in borders([name[i] for i in 0..n]):
        wait += Z(m) ** b
    print(f"{wait.v:04}")
```

### 5.2 NOI 2015 — Wine Tasting (`noi15p5`)

For every `r in 0..n`, count pairs of suffixes sharing at least `r` letters and
find the maximum product of their scores. Sort suffixes, then merge adjacent
ranks from largest LCP down using `DsuOf`.

```cp
// Two suffixes are r-alike iff every adjacent LCP between their ranks is >= r.
// Sweep r downward, merging adjacent ranks and tracking each group's min/max score.
use std.strings.{SuffixArray}
use std.dsu.{DsuOf}

struct Span(lo: Int, hi: Int)

let n: 1..=300_000 = read()
let s: Str = read()
let score: [0..n]Int = read()
let sa = SuffixArray(s)

domain Rank = 0..n
var groups = DsuOf<Rank, Span>(
    r => Span(score[sa.sa[r.to_idx]], score[sa.sa[r.to_idx]]),
    (a, b) => Span(min(a.lo, b.lo), max(a.hi, b.hi)),
)

var (pairs, best): (Int, ?Int) = (0, None)
let order = sorted(0..n - 1, by: j => -sa.lcp[j])
var at = 0
var ans: List<(Int, Int)> = []
for r in (0..n).rev:
    while at < order.len and sa.lcp[order[at]] >= r:
        let j = order[at]
        at += 1
        let (u, v) = (Rank(j), Rank(j + 1))
        let (su, sv) = (groups.get(u), groups.get(v))
        pairs += groups.size(u) * groups.size(v)
        best max= max(su.lo * sv.lo, su.lo * sv.hi, su.hi * sv.lo, su.hi * sv.hi)
        groups.unite(u, v)
    ans.push((pairs, best ?? 0))
for (cnt, prod) in ans.rev: print(cnt, prod)
```

### 5.3 NOI 2016 — Excellent Splits (`noi16p1`)

Count substrings of the form `AABB` (`A`, `B` non-empty). For each half-length
`w`, every square `AA` covers a unique checkpoint pair `(q, q + w)` with `q` a
multiple of `w`; its valid start range is determined by `lcs(q, q + w)` and
`lcp(q, q + w)`.

```cp
// Every square AA of half-length w straddles a unique (q, q + w) with w | q.
// lcs and lcp give the exact interval of starts; difference arrays accumulate
// squares starting and ending at each position.
use std.strings.{RollingHash}
use std.seq.{scan}

@!multitest
let s: Str = read()
let n = s.len
let h = RollingHash(s)
var starts = List.filled(n + 1, 0)
var ends = List.filled(n + 1, 0)
for w in 1..=n / 2:
    for k in 0..(n - 1) / w:
        let q = k * w
        let lo = max(q - h.lcs(q, q + w), q - w + 1)
        let hi = min(q, q - w + h.lcp(q, q + w))
        if lo > hi: continue
        starts[lo] += 1
        starts[hi + 1] -= 1
        ends[lo + 2 * w - 1] += 1
        ends[hi + 2 * w] -= 1
let (begun, ended) = (scan(starts, +), scan(ends, +))
print(sum(ended[i] * begun[i + 1] for i in 0..n - 1))
```

### 5.4 SAC 2022 CC3 P5 — Multi-Pattern Dictionary (`sac22cc3p5`)

Given `n` case-insensitive words and `m` texts, print for each text how many
distinct dictionary words appear in it.

```cp
// Build one Aho-Corasick automaton over lowercase letters; for each text,
// count how many patterns have a non-zero occurrence count.
use std.strings.{AhoCorasick, lower}

domain Letter = 'a'..='z'
fn to_letters(s: Str) -> List<Letter> = [Letter(c.lower) for c in s]

let (n, m): (Int, Int) = read()
let words = [to_letters(read::<Str>()) for _ in 0..n]
let ac = AhoCorasick<Letter>(words)
repeat m:
    let text = to_letters(read::<Str>())
    print(count(c > 0 for c in ac.counts(text)))
```

### 5.5 Palindrome Factorization (CF 932G / Palindromic Partition DP)

Minimum number of palindromic pieces to partition a string `s` of length `n`
(`n <= 5 * 10^5`). `PalindromicTree<Letter>` maintains series links so every
palindromic suffix of `s[..=i]` is grouped into `O(log n)` arithmetic
progressions of lengths, giving O(n log n) DP.

```cp
// Group palindromic suffixes by arithmetic progression via series_link:
// series_dp[v] caches the minimum dp over v's progression, updated in O(1).
use std.strings.{PalindromicTree}

domain Letter = 'a'..='z'
let s: Str = read()
let n = s.len
var pt = PalindromicTree<Letter>()
var dp = List.filled(n + 1, n + 1)
var series_dp = List.filled(n + 3, n + 1)
dp[0] = 0

for i in 0..n:
    var v = pt.push(Letter(s[i]))
    while pt.pal_len(v) > 0:
        series_dp[v] = dp[i + 1 - pt.pal_len(pt.series_link(v)) - pt.diff(v)]
        if pt.diff(v) == pt.diff(pt.link(v)):
            series_dp[v] min= series_dp[pt.link(v)]
        dp[i + 1] min= series_dp[v] + 1
        v = pt.series_link(v)
print(dp[n])
```

## 6. Open questions and core dependencies

1. **Sequence trait for `[D]T` (`T3`).** Today `z_function`, `prefix_function`,
   `manacher`, and `SuffixArray` take `List<T>` (which `Str` satisfies), so a
   domain-indexed `[D]Int` still copies via `[a[i] for i in D]`. Once the core
   provides `Seq<T>` / `Indexed<I>`, these functions widen to accept `[D]T`
   directly with zero call-site breakage.
2. **Nominal `Rank` domain on `SuffixArray`.** Today `SuffixArray.sa` and
   `rank` use `Int` because structs cannot yet hold a fresh existential domain
   field (`domain Rank`). Keeping `sa`, `rank`, and `lcp` as `List<Int>` works
   today and composes cleanly with `domain Rank = 0..n` in user code.
3. **Single-mod `2^61 - 1` hashing (`T2`).** Once `Mod<P>` uses `mul_mod` for
   `P > 3 * 10^9`, `HashVal` could optionally narrow from two 31-bit halves to
   one 61-bit residue, halving register pressure while keeping the same API.

## Coverage

Audit against AtCoder Library (`string`), KACTL (`strings`), cp-algorithms (`string`), and strong contestant templates (Benq, jiangly, ecnerwala, tourist). Status: `have` (implemented body in `spec/std/strings.cp`), `new` (`= todo` spec addition), `out` (intentionally omitted).

| Reference item | Source(s) | Status | Our name / reason if omitted |
| --- | --- | --- | --- |
| Basic `Str` ops (`repeat`, `starts_with`, `ends_with`, `find`, `rfind`, `count`) | stdlib | have | `repeat`, `starts_with<S, P>`, `ends_with<S, P>`, `find`, `rfind`, `count` (`starts_with`/`ends_with` generalized to `Indexed`) |
| ASCII `Char` helpers (`lower`, `upper`, `digit`, predicates) | Contest basics | have / new | `Char.{lower,upper,digit}` (have); `Char.{is_digit,is_lower,is_upper,is_alpha,is_alnum,is_space}` (new) |
| `Str` utilities (`to_lower`, `to_upper`, `trim`, `split`, `split_first`, `split_whitespace`, `join`, `replace`, `parse_int`) | Contest I/O & parsing | new | `Str.{to_lower,to_upper,trim,split,split_first,split_whitespace,join,replace,parse_int}` |
| Z-algorithm | ACL `z_algorithm`, KACTL `Zfunc`, cp-alg | have | `z_function<S: Indexed>` |
| Prefix function / KMP | KACTL `KMP`, cp-alg | have | `prefix_function`, `find_all`, `borders`, `prefix_matches`, `overlap`, `period` |
| KMP prefix automaton (O(1) DP transitions) | cp-alg, Benq | new | `KmpAutomaton<A>.{new,len,step,link,find_all}` |
| Shift-And / bitset matching | Contest templates | new | `shift_and<S: Indexed, P: Indexed>` |
| Wildcard pattern matching (`?`) | cp-alg, contest templates | new | `find_all_wildcard<S: Indexed, P: Indexed>(..., wildcard: Char = '?')` |
| Longest common prefix / suffix of two sequences | Contest basics | have | `common_prefix_len`, `common_suffix_len` |
| Palindrome check + Manacher | KACTL `Manacher`, cp-alg | have | `is_palindrome`, `manacher`, `Palindromes.{is_palindrome,longest_at,longest_between,longest,count}` |
| Lyndon factorization (Duval) | cp-alg | have | `lyndon_factorization<S: Indexed>` |
| Minimum / maximum cyclic rotation | KACTL `MinRotation`, cp-alg | have / new | `min_rotation` (have), `max_rotation` (new) |
| Main-Lorentz / Bannai runs (all maximal repetitions) | cp-alg, Bannai | have | `lyndon_runs<S: Indexed>` |
| Suffix array (SA-IS) + Kasai LCP | ACL `suffix_array`/`lcp_array`, KACTL `SuffixArray`, cp-alg | have | `suffix_array<S: Indexed>(s) -> SuffixArray`, `SuffixArray.{sa,rank,lcp,len,is_empty,lcp_between,rank_range,find_all,count,lcp_range,compare,distinct}` |
| Suffix array rank-LCP, prefix-bucket split, k-th distinct substring | KACTL, jiangly, ecnerwala | new | `SuffixArray.{lcp_of_ranks,split,kth_distinct}` |
| Compressed suffix tree (from SA + LCP) | KACTL `SuffixTree`, ecnerwala (`ucup5s05f`) | new | `SuffixTreeEdge`, `suffix_tree_edges<S: Indexed>` |
| Double-mod polynomial rolling hash (1D) | cp-alg, Benq, jiangly | have | `HashVal`, `Hashable`, `RollingHash.{new(s, bases:),len,all,slice,get,combine,lcp,lcs,lcp_with}` |
| Mersenne `2^61 - 1` rolling hash (1D) | KACTL `Hashing` (mod `2^64-1`/Mersenne), Benq | new | `RollingHash61.{new(s, base:),len,all,slice,get,combine,lcp,lcs,lcp_with}` |
| Dynamic rolling hash (point update + range hash, forward & reverse) | Contest templates (SegTree/Fenwick hash) | new | `DynamicHash<D>.{new(seq, bases:),len,is_empty,all,get,set,rev_get,is_palindrome}` |
| 2D rolling hash (grid sub-rectangles) | cp-alg, contest templates | have | `Hash2D<R, C>.{new,rect}` |
| Suffix automaton (SAM) + online/generalized + k-th substring | cp-alg, Benq, jiangly | have / new | `SuffixAutomaton<A>.{new,of,push,root,len,step,link,longest,shortest,first_pos,find,contains,distinct,topo_order,occ_counts,lcs}` (have); `{reset_last,push_seq,kth_substring}` (new) |
| Palindromic tree (eertree) + series_link + deque/rollback/min-partition | cp-alg, Benq, jiangly | have / new | `PalindromicTree<A>.{new,of,push,odd_root,even_root,len,pal_len,link,step,diff,series_link,occ_counts}` (have); `{push_front,pop,last_state,contains,min_palindrome_partition}` (new) |
| Prefix trie + order statistics | cp-alg, contest templates | have / new | `Trie<A>.{new,root,len,step,find,add,remove,count,count_prefix,ends_at,pass_at}` (have); `{contains,clear,kth,count_less}` (new) |
| Binary XOR trie (max/min XOR, count `< limit`, k-th XOR) | Benq, jiangly, cp-alg | out | Owned by `std.bitset` (`std.bitset.XorTrie`). |
| Aho-Corasick (static) | KACTL `AhoCorasick`, cp-alg | have / new | `AhoCorasick<A>.{new,root,len,patterns_len,step,link,hit,words_at,find_all,counts}` (have); `{fail_order,exit_link,pattern_state}` (new) |
| Dynamic / online Aho-Corasick (log-decomposed buckets) | cp-alg, contest templates | new | `DynamicAhoCorasick<A>.{new,add,remove,count_matches}` |
| `HashInterval` single-mod `2^64` hash | KACTL `Hashing` | out | Omitted because `RollingHash` (double 31-bit salted) and `RollingHash61` (Mersenne `2^61 - 1` salted) cover both fast-32-bit and single-64-bit hashing without hackable power-of-2 mod (Thue-Morse collision). |
| Ukkonen online suffix tree | cp-alg | out | Omitted because `SuffixAutomaton<A>` covers online suffix-structure queries in O(1) amortized per char, and `suffix_tree_edges` builds the compressed suffix tree offline in O(n) from SA+LCP. |
| Rabin-Karp standalone function | cp-alg | out | Omitted because `find_all` (KMP, deterministic O(n+m)) and `RollingHash` already cover single-pattern and hash-based matching. |
| Edit distance / LCS DP | cp-alg | out | Omitted because it is a 10-line problem-specific DP whose weights/tie-breaking/path-recovery vary per problem. |
| Suffix array of a tree / persistent trie / persistent SAM | Niche | out | Omitted because tree-SA and persistent automata have problem-specific node payloads; users compose `Trie`/`SuffixAutomaton` or explicit node pools. |
| Unicode / regex / locale | General-purpose | out | Out of scope for competitive programming (ASCII bytes and generic `Indexed`/`Seq` sequences). |

### Renames and removals from `stdlib/strings.cp`

- **`RollingHash.with_bases(s, bases)` -> removed / folded into `RollingHash.new(s, bases: ?(Int, Int) = None)`:** One constructor with an optional named argument `bases:` instead of two constructors (`new` + `with_bases`), matching `Hash2D.new(g, bases:)`, `RollingHash61.new(s, base:)`, and `DynamicHash.new(s, bases:)`.
- **`starts_with` / `ends_with`:** Generalized from `(Str, Str)` to `<S: Indexed, P: Indexed>` so `List<T>`, `[D]T`, `StrView`, and `ListView` share the same name with zero copies.

## Cross-area

- **`std.seq`:** `count`, `find`, `rfind`, `starts_with`, `ends_with` overlap conceptually with sequence queries on `List<T>`/`[D]T`. Keeping `starts_with`/`ends_with` generic over `Indexed` avoids duplication; `Str.count(c: Char)` / `Str.find(p)` / `Str.rfind(p)` remain `Str`-specific because they accept a `Str` or `Char` pattern.
- **`std.math` / `std.poly`:** `find_all_wildcard` can also be implemented via NTT for large alphabets (O(n log n) vs O(n * |pat| / 64)); the strings spec exposes the single name `find_all_wildcard`, independent of backend.
- **`std.bitset`:** Binary `XorTrie` is owned by `std.bitset` rather than `std.strings`; `std.strings.Trie<A>` covers general-alphabet prefix tries.
- **`std.segtree`:** `DynamicHash` is a monoid-over-segment-tree hash specialized for string range hashing and palindrome checks; it avoids making users wire up custom `HashMonoid` structs and power tables manually.

## Powers

Every power from `spec/core/12-powers.md` (and any gap beyond it) needed by `spec/std/strings.cp`:

1. **W4 (Sequences, views, and index spaces — `Indexed`, `Seq`, `Range<D>`, cut ranges):**
   - Signatures: `starts_with`, `ends_with`, `z_function`, `prefix_function`, `find_all`, `borders`, `prefix_matches`, `overlap`, `lyndon_factorization`, `min_rotation`, `max_rotation`, `lyndon_runs`, `manacher`, `suffix_array`, `SuffixArray.{rank_range,find_all,count}`, `suffix_tree_edges`, `RollingHash.{new,get}`, `RollingHash61.{new,get}`, `DynamicHash.new`, `SuffixAutomaton.{of,find,contains,lcs,push_seq}`, `PalindromicTree.{of,contains}`, `Trie.{find,add,remove,count,count_prefix,contains,count_less}`, `AhoCorasick.{new,find_all,counts}`, `DynamicAhoCorasick.{add,remove,count_matches}`.
   - Smallest example: `let sa = suffix_array(a)` where `a: [N]Int`, and `h.get(l..r)` where `l, r: N`.
2. **W6 (Generators / `yields`):**
   - Signatures: `find_all`, `borders`, `KmpAutomaton.find_all`, `shift_and`, `find_all_wildcard`, `SuffixArray.find_all`, `AhoCorasick.find_all`.
   - Smallest example: `for (at, pat) in ac.find_all(s): if pat == target: break`.
3. **W8 (User places — `get`/`set`):**
   - Signatures: `DynamicHash.{get,set}` (so `dh[i] = c` updates the hashed element in O(log n) and `dh[l..r]` queries the range hash).
   - Smallest example: `dh[i] = 'a'; if dh.is_palindrome(l..r): ans += 1`.
4. **W12 (Cost on first `///` line):**
   - Signatures: All 191 public declarations in `spec/std/strings.cp`.
5. **Defaulted named arguments (Decision 134 / `spec/delta/parser-gaps.md`):**
   - Signatures: `RollingHash.new(s, bases: ?(Int, Int) = None)`, `RollingHash61.new(s, base: ?Int = None)`, `DynamicHash.new(s, bases: ?(Int, Int) = None)`, `find_all_wildcard(self, pat, wildcard: Char = '?')`.
   - Smallest example: `let h2 = RollingHash.new(t, bases: h1.bases)`.
6. **Gaps beyond `spec/core/12-powers.md`:**
   - **Existential / struct-owned domains** (for `SuffixArray` rank domain `sa.Rank` and `SuffixAutomaton`/`PalindromicTree`/`AhoCorasick` state domains): today states and SA ranks are plain `Int`, so `sam.step(v, c)` and `sa.lcp_of_ranks(l, r)` cannot statically distinguish state IDs from string positions. Smallest example: `struct SuffixArray<T>: domain Rank = 0..s.len; sa: [Rank]Int`.
   - **Constrained alphabet strings `Str of "01?"`** (Decision 134 / `spec/delta/parser-gaps.md`, used by `cf2187d`): syntax-level refinement type rather than a `std.strings` API.
   - **128-bit modular multiplication primitive (`mul_mod` / `u128`)** for `RollingHash61` (`mod 2^61 - 1`): needed in user/stdlib CP Lang so `RollingHash61` can be written in ordinary CP Lang without compiler privilege.

## Dreams

All dreams in `proto/ports/*/dream/` touching strings:

- `proto/ports/ecnerwala/dream/cf1943b.cp`: uses `std.strings.manacher` (`Palindromes.is_palindrome`) — in spec (`have`), passes.
- `proto/ports/ecnerwala/dream/cf2124d.cp`: uses `std.strings.is_palindrome` — in spec (`have`), passes.
- `proto/ports/ecnerwala/dream/cf1909g.cp`: uses `std.strings.{common_prefix_len, common_suffix_len}` — in spec (`have`), passes.
- `proto/ports/ecnerwala/dream/ucup5s05f.cp`: imports `std.strings.suffix_tree_edges` — **now in spec (`new`)**; both `std.strings.suffix_tree_edges` and `suffix_tree_edges` errors are resolved (remaining error in `ucup5s05f.cp` is `std.poly.SubproductTree`).
- `proto/ports/ecnerwala/dream/cf2187d.cp`: uses `Str of "01?"` (language syntax, Decision 134 in `spec/delta/parser-gaps.md`) and `s.split_first('?')` (**now in spec (`new`)**).
