# Indices and keys

| | |
|---|---|
| Kind | Language design |
| Owner | spec |
| Leads | `spec/core/04-indices.md`, `spec/core/04a-identity-vs-offset.md`, `spec/core/04b-keys.md` |
| Describes | `spec/core/04-indices.md`, `spec/core/04a-identity-vs-offset.md`, `spec/core/04b-keys.md`, `spec/conformance/core-04/` |

How CP Lang names places in a sequence: domains, positions, cuts, ranges,
and the keys a slice keeps. Terms follow `spec/GLOSSARY.md`: a *domain*
is a finite index space, a *position* is an element of it, a *cut* is a
gap between positions, a *piece* is the positions between two cuts, and a
*key* is a domain value that indexes a sequence.

## The problem

Index bugs are the most common contest wrong answer, and they come from
integers doing three jobs at once. In C++:

```cpp
vector<int> a(n + 1);            // 1-based? 0-based? the type doesn't say
for (int i = 1; i <= n; i++) ... // off by one at either end
int j = lower_bound(a.begin() + l, a.begin() + r, x) - a.begin() - l;
                                 // an offset into the slice, used as an index into a
adj[u].push_back(w);             // u and w both int: swapped arguments compile
```

In Python, `s[2:][k]` and `s[k]` look alike but mean different elements:
the slice renumbers from zero, so an index found in a slice silently
points somewhere else in the original. Both languages also blur "element
\(i\)" with "the gap before element \(i\)", which is where every
off-by-one in a binary search or prefix sum lives.

## The model

```cp
domain V = 1..=5
let a = [V] v => Int(v)
let i = V.cuts(4)                // the cut after the fourth position
var best = 0
for j in ..i: best = max(best, a[j..i].sum())    // j..i is a piece
print(best)

let s = [1, 2, 3, 4, 5, 6]
let k = 3
print(s[2..][k], s[2..].nth(1))  // a slice keeps s's keys; .nth counts from its start
```

What a contestant holds in their head:

1. **A domain is a type of index.** `domain V = 0..n` makes `V` a type;
   `[V]Int` is an array indexed only by `V`. Vertices and edges can't be
   mixed up.
2. **Positions and cuts are different things.** A position names an
   element; a cut names a gap. `.l` and `.r` move from one kind to the
   other, `.prev` and `.next` stay in the same kind.
3. **A range `l..r` holds values of its endpoints' kind.** Between two
   cuts it is a piece of positions.
4. **A slice keeps its source's keys.** `s[2..][3]` is `s[3]`. Nothing
   renumbers unless you write `.rebase` or `.rekey`.
5. **Keys in, keys out.** A library fn given a slice returns keys of the
   source; combining two sequences pairs them by walk order, not key.

## Key decisions

### Domains are types; positions are not integers

**Chose:** a domain is a finite ordered type; its values are positions,
and arrays are indexed by a domain (decision 137; R04.2.1 to R04.2.28).
`Int(v)` reads the integer back; position arithmetic gives an `Int`, not
a position.

```cp
domain V = 1..=4
let v = V(2)
print(Int(v.next!) * 10 + Int(v.prev!))
print(Int(V(4).next ?? V(1)))
```

**Lost:** plain integer indices (C++, Python), and newtype indices
written by hand (Rust's `struct NodeId(u32)`: correct but heavy, so
nobody does it in a contest). Why: "types carry the proofs": an index
into `[V]Int` is a `V`, and the swap of `u` and `w` above is a type
error.

### Cuts are their own kind

**Chose:** a domain has positions and cuts; a binary search returns a
cut, `V.cuts(k)` is the cut after \(k\) positions, `.l` and `.r` give
the positions on either side as optionals (decision 137; chapter 4,
"Positions and cuts move the same way").

```cp
let c = (0..10).bsearch(x => x * x >= 50)
print(Int(c))
domain V = 1..=4
let d = V.cuts(1)
print(Int(d.l!) * 10 + Int(d.r!))
```

**Lost:** half-open integer conventions only (C++ iterators), where "the
answer is position \(i\)" and "the answer is just before \(i\)" are both
`int`. Why: `lower_bound` versus `upper_bound` versus `-1` bugs are
exactly the confusion of the two kinds.

### One range type; the piece literal

**Chose:** `Range<D>` holds values of its endpoint kind: a range of
cuts walks cuts, a range of positions walks positions. One exception in
how a literal is typed: a half-open `j..i` between cuts becomes a
`Range<V>` of positions when a piece is expected, as in `a[j..i]`
(chapter 4, "One exception, in how a literal is typed: the piece";
R04.4.1 to R04.4.32). A range can be empty without trapping.

**Lost:** separate range types per kind (more names to learn), and
ranges that trap when `l > r` (they'd make every empty-window edge case
a crash). Why: the common loop `for j in ..i: a[j..i]` reads as written.

Also lost (decision 137): every range as positions with a separate
`r.cuts`. A cleaner model, but `for j in lo..i: dp[i] min= dp[j] +
cost(j..i)` stops type-checking. And the names `.n` and `.p` for
`.next` and `.prev`, which read as count and parent.

### Identity everywhere: slices keep keys

**Chose:** a slice, and every view (`.map`, `.rev`, `.drop`, a grid
row), is indexed by the keys of its source. Chapter 4a scored three
options across eight kinds of contest use (order-only, index found in a
slice used outside, outside index used on a slice, counters and
literals, generic algorithms, grids, revision, other views) and chose
identity everywhere (decision 143; R04a.11.1 to R04a.11.6).

| Option | `s[2..][k]` | Failure mode |
|---|---|---|
| Offset (Python, C++ `span`) | `s[k + 2]` | an index found in the slice, used on `s`, is silently wrong |
| Identity (chosen) | `s[k]` | a counter from 0 into a slice traps in checked builds and is linted |
| Hybrid (identity for arrays, offset for strings) | depends | two rules to remember |

**Why:** offset rebasing fails *silently* (a wrong answer), identity
fails *loudly* (a trap at the line, and a lint at compile time when the
index is a literal). Relative access is still one word:

```cp
let s = [1, 2, 3, 4, 5, 6]
let k = 3
print(s[2..][k])
print(s[2..].nth(k))
```

```cp
let s = [5, 6, 7, 8]
let w = s[1..3]
print(w.rebase[0], w.rebase[1])          // explicit zero-based view
```

A range's keys are its values, so `(3..8)[5]` is `5`, never the sixth
element: positional `r[k]` lost (decision 289; Ryan: "r[3] should be
equal to 3"), and the relative element is `r.nth(k)`.

### Keys in, keys out; pairing by order

**Chose:** every library fn that returns an index returns a key of the
sequence it was given, so a search in a suffix answers in the source's
keys (chapter 4b rules; R04b.1.1 to R04b.1.12). Operations that combine
two sequences (`==`, `zip`, `starts_with`, concatenation) pair elements
by walk order, never by key (R04b.1.3).

```cp
let s = "a,b,c"
print(s[2..].search(',')!)               // 3: a key of s

let a = [1, 2, 3, 4]
var t = 0
for (x, y) in zip(a[2..], a[..2]): t += x * y
print(t)
```

**Lost:** pairing by key (two slices of one array at different offsets
could never be compared). Why: order is what "these two windows are
equal" means; keys are what "where is it" means.

### Materializing renumbers; `.first`, `.last`, `.nth`

**Chose:** `List(view)` and `Str(view)` build a new value with new keys
from zero (R04b.1.6); `.collect()` keeps keys. `.first`, `.last` and
`.nth(k)` give ends and relative elements of any view, and a range's
keys are its values (decision 289). `.rekey(r)` gives a view new keys.

```cp
let s = "hello"
let t = Str(s[1..4])
let i = 0
print(t[i])

let w = [4, 5, 6].rekey(10..13)
print(w[11])
```

The identity rule's loud failure, a key from before the copy:

```cp-fragment
let a = [10, 20, 30, 40]
let c = List(a[2..])
print(c[2])                              // traps R0101: c has keys 0 and 1
```

## Worked examples

**Maximum subarray ending at a cut**, using pieces (chapter 4, piece
literal):

```cp
domain V = 1..=5
let a = [V] v => Int(v)
let i = V.cuts(4)
var best = 0
for j in ..i: best = max(best, a[j..i].sum())
print(best)
```

**Comparing windows by order** (chapter 4a §A):

```cp
let s = "abcabc"
if s[0..3] == s[3..6]: print("eq")
if s[3..].starts_with(s[0..2]): print("starts")
```

**A view keeps keys through `map`** (chapter 4a §H):

```cp
let a = [10, 20, 30, 40]
let m = a[1..3].map(x => x * 2)
let i = 2
print(m[i])
```

# Reference

## Where the precise rules are

| Topic | Spec | Rule ids |
|---|---|---|
| The model | `spec/core/04-indices.md`, "The model in one paragraph" | |
| Domains | chapter 4, "Domains" | R04.2.1 to R04.2.28 |
| Positions and cuts | chapter 4, "Positions and cuts move the same way" | R04.3.1 to R04.3.3 |
| Ranges, the piece, open ranges, methods | chapter 4, "Ranges" | R04.4.1 to R04.4.32 |
| Arrays over domains, empty domains | chapter 4 | R04.7.1 to R04.8.10 |
| Decision 137, alternatives | chapter 4, "Decided (decision 137)", "Why, and the alternatives" | |
| Identity versus offset, the tally | `spec/core/04a-identity-vs-offset.md` §A to §H, "Tally" | R04a.2.1 to R04a.9.7 |
| Identity everywhere | 04a, "Recommendation" | R04a.11.1 to R04a.11.6 |
| Keys in, keys out | `spec/core/04b-keys.md`, "The rules" | R04b.1.1 to R04b.1.12 |
| Contest code, helpers, library code | 04b | R04b.3.1 to R04b.5.9 |

Decisions: 2, 137, 143, 162, 164, 246, 289, 291 in `decisions.md`.

## Changes

- 2026-11-23: first version.
