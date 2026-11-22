# Round 7: sequence patterns

Status: proposal for review. Scope: the everyday sequence problems (runs,
windows, two pointers, nearest smaller, contribution counting, prefix
tricks, adjacent pairs, circular arrays, grouping) and how a few general
mechanisms cover all of them. Builds on round 2 (views keep keys, folds,
scans) and round 5 (op values, laws, `Window(op)`).

## The model in one paragraph

**Every pattern is a way to cut a sequence into pieces, plus a fold over
each piece.** Pieces are views that keep the source's positions (round
2), so an answer is always a position or a range of the input, never an
offset you translate back. How the pieces are cut is one of five shapes:
fixed length (`windows`, `chunks`), equal or linked neighbours (`runs`),
separators (`split`), the longest or shortest piece satisfying a
condition at each end (`maximal`, `minimal`), and the piece where an
element wins (`spans`). How each piece is summarized is an op value from
round 5, and the op's laws pick the O(1)-per-step strategy, so "sliding
min", "sliding sum" and "sliding gcd" are one call with different ops.
State that isn't a fold (distinct counts, a frequency table) implements
one small protocol, `Track`, which two pointers and Mo's algorithm both
accept.

## 1. Mechanisms

| # | Mechanism | Forms | Where it comes from |
|---|---|---|---|
| 1 | pieces are views with keys | `w.keys`, `w.len`, `a[w]` | round 2 |
| 2 | cutting into pieces | `windows`, `chunks`, `runs`, `split`, `maximal`, `minimal`, `spans` | this doc |
| 3 | folding a piece | `w.fold(op)`, `w.sum()`, `w.max(by:)`; `pieces.fold(op)` as a sliding fold | round 5 |
| 4 | prefix folds over cuts | `a.scan(op)`, `pre[r]` | round 2 |
| 5 | state that grows and shrinks | `Track<T>`: `push(x)`, `pop(x)` | this doc |
| 6 | nearest neighbour by order | `prev_smaller`, `next_smaller`, `prev_same`, `next_same` | this doc |

Everything else is an idiom over these six, shown in section 3.

## 2. The pieces

All of these are O(1) to make and lazy, read without parentheses only when
they are views (`a.pairs`), and yield views of `a` keyed by `a`'s
positions. Each works on any `Indexed` (`[D]T`, `List`, `Str`, a slice).

| Form | Pieces | Cost to walk |
|---|---|---|
| `a.windows(k)` | every length-`k` piece, left to right; none if `k > a.len` | O(n) |
| `a.chunks(k)` | consecutive length-`k` pieces; the last may be shorter | O(n) |
| `a.runs(by: key)` | maximal pieces of equal `key` | O(n) |
| `a.runs(link: (x, y) => Bool)` | maximal pieces where every adjacent pair links | O(n) |
| `a.split(sep: p)` | pieces between elements where `p` holds (separators excluded; empty pieces kept) | O(n) |
| `a.maximal(state, ok:)` | for each right end, the longest piece ending there that is `ok` | O(n) amortized |
| `a.minimal(state, ok:)` | for each right end, the shortest piece ending there that is `ok`, or none | O(n) amortized |
| `a.spans(by: key)` | for each position, the widest piece where it is the minimum | O(n) |

```python
s.runs()                              // "aabccc" -> "aa", "b", "ccc"
a.runs(link: (x, y) => x < y)         // strictly increasing runs
a.runs(link: (x, y) => y - x <= d)    // gaps of at most d
line.split(sep: c => c == ' ')        // words, as pieces of line
```

`runs(by:)` is `runs(link: (x, y) => key(x) == key(y))`; it stays as its
own option because equal-key runs are the common case and need no lambda.

### Folding pieces: the sliding fold

A sequence of pieces has `.fold(op)`, which yields one value per piece.
On `windows(k)` it is the sliding fold, O(1) amortized per window, with
the strategy chosen by the op's laws exactly as `Window(op)` chooses it:

| Law of `op` | Strategy | Cost per step |
|---|---|---|
| `Group` (`+`, `^`, `Mod` `*` without zero) | add the new element, subtract the old | O(1) worst case |
| `Selective` (`min`, `max`, `min(by:)`) | monotone deque | O(1) amortized |
| any `Op` (`gcd`, matrix product, `MaxSub`) | two stacks | O(1) amortized |

```python
a.windows(k).fold(+).max()!           // best sum of k consecutive
a.windows(k).fold(min)                // sliding minimum, one per window
a.windows(k).fold(gcd).count(g => g > 1)
```

On other pieces (`runs`, `chunks`, `split`) `.fold(op)` folds each piece
directly, which is already O(n) total since they don't overlap. Writing
`for w in a.windows(k): w.sum()` is O(nk); the lint `window_refold`
suggests `.fold(+)`.

This replaces `sliding_min`, `sliding_max`, `sliding_fold`, `MonoDeque`
used for fixed windows, and the hand-written two-stack queue.

### Two pointers: `maximal` and `minimal`

Two pointers is a search for, at each right end, the extreme left end
where a condition holds. It needs one fact about the condition:

- `maximal` needs `ok` **hereditary**: if a piece is ok, every piece inside
  it is ok (sum at most K over non-negative values, at most k distinct).
- `minimal` needs `ok` **upward closed**: if a piece is ok, every piece
  containing it is ok (sum at least S, contains every colour).

Both walk `r` left to right, move `l` only forward, and keep `state` in
step by calling `state.push(a[r])` and `state.pop(a[l])`. They yield one
`Range<D>` per right end (`minimal` yields `?Range<D>`).

```python
pub trait Track<T>:
    fn push(mut self, x: T)       // x joins at the right end
    fn pop(mut self, x: T)        // x leaves at the left end (the oldest)

pub fn maximal<D, T, S: Track<T>>(self: [D]T, mut state: S, ok: S -> Bool)
    -> yields Range<D>
```

`Track` implementations that already exist:

| Type | `push` / `pop` | Typical `ok` |
|---|---|---|
| `Window(op)` | fold queue | `w.all <= k` |
| `Counter<K>` | count up / down; `.distinct` kept | `c.distinct <= k` |
| `Bag<T>` (sorted multiset) | insert / erase one | `b.max! - b.min! <= d` |
| `track(add:, remove:)` | your two closures | your own predicate |

```python
let longest = a.maximal(Window(+), ok: w => w.all <= K).max(by: .len)!
let atmost_k = a.maximal(Counter<C>(), ok: c => c.distinct <= k)
let count = atmost_k.map(.len).sum()           // subarrays with <= k distinct
let shortest = a.minimal(Window(+), ok: w => w.all >= S).present().min(by: .len)
```

- **The promise is checked where it is cheap.** Hereditary and upward
  closed are promises like round 5's laws. `cpl stress` compares
  `maximal`/`minimal` against their O(n²) definition on small inputs, so
  a non-monotone `ok` shows up as a counterexample naming the piece.
- **Mo's algorithm takes the same protocol.** `mo(queries, state, answer:)`
  with `state: Track<T>` replaces `add:`/`remove:` closures; Mo pushes and
  pops at both ends, so it needs a state whose answer ignores order
  (`Counter`, `Bag`), and `Window(op)` there needs `Commutative`.
  `track(add:, remove:)` keeps the closure form one call away.
- **The moving window over positions** (`Window<D>(enter, leave)` today,
  for divide-and-conquer DP costs) becomes `Sweep(state)` with
  `s.move(to: r)`, again over a `Track`. This frees the name `Window` for
  round 5's fold queue.

### Contribution counting: `spans`

"Sum of minimums over all subarrays", "largest rectangle in a histogram"
and "count subarrays where `a[i]` is the maximum" all need, for each `i`,
the widest piece in which `a[i]` is the minimum. `a.spans(by: key)` gives
it as `[D]Range<D>`, ties broken so every subarray is counted once (strict
on the left, non-strict on the right). Maximum is `by: desc`.

```python
let sp = a.spans()
let total = a.keys.map(i => a[i] * (sp[i].l..i.r).len * (i.l..sp[i].r).len).sum()
let rect = a.keys.map(i => a[i] * sp[i].len).max()!     // histogram
```

This replaces `max_histogram` and covers the contribution technique with
one structure; `cartesian_tree` stays, since it is a tree.

### Nearest by order

| Form | Result |
|---|---|
| `a.prev_smaller(by:, strict: true)` | `[D]?D`: nearest earlier position with a smaller key |
| `a.next_smaller(by:, strict: true)` | `[D]?D`: nearest later one |
| `a.prev_same(by:)`, `a.next_same(by:)` | `[D]?D`: nearest earlier/later position with an equal key |

Larger is `by: desc` (round 5's one way to flip an order), so
`prev_less`, `prev_greater`, `next_less`, `next_greater` merge into two
names. `prev_same` is the "last occurrence" array behind distinct-count
queries. `MonoStack` stays for custom stacks (rollback on trees).

## 3. Idioms over the mechanisms

No helper is added for these; each is one or two lines.

| Problem | Code |
|---|---|
| run-length encoding | `[(r.first!, r.len) for r in s.runs()]` |
| longest run | `s.runs().max(by: .len)!.len` |
| adjacent differences | `a.pairs.map((x, y) => y - x)` |
| count adjacent pairs with a property | `a.pairs.count((x, y) => x > y)` |
| best subarray sum (Kadane) | `a.fold(MaxSub)` (round 5's summary; the same op answers range queries in a `SegTree`) |
| maximum and how often | `a.fold(MaxCount)` |
| prefix records | `a.keys.filter(i => a[i] > pre[i.l])` with `pre = a.scan(max)` |
| count subarrays with sum `K` | `var seen = Counter<Int>()`; `for c in pre.keys: ans += seen[pre[c] - K]; seen.push(pre[c])` |
| range sums | `pre = a.scan(+)`; `pre[r.r] - pre[r.l]` |
| first position where a prefix fold passes `x` | `pre.keys.first_true(c => pre[c] >= x)` (monotone) or `.first(...)` |
| polygon edges | `poly.cyclic_pairs` |
| circular windows | `a.ring.windows(k)` (below) |
| group equal keys anywhere | `a.group(by: k)`, `a.tally(by: k)`, `buckets(key)` |
| dedup | `a.dedup()` (adjacent), `a.distinct(by:)` (anywhere) |
| merge sorted sequences | `merge(a, b, by:)` yields in order; `kway_merge(lists, by:)` |
| inversions, LIS | `a.inversions()`, `a.lis(strict:)` |

### Circular sequences: `a.ring`

`a.ring` is a view of `a` read as a cycle: `a.ring.windows(k)` yields the
`n` windows starting at each position (wrapping), `a.ring.from(i)` walks
all `n` positions starting at `i`, and every element is still a position
of `a`. It replaces the "double the array" trick, which costs O(n) memory
and loses the positions. A wrapping window isn't one range of `a`, so its
`.keys` is a sequence of positions, not a `Range<D>`.

## 4. How the parts stay coherent

- **Same answer type everywhere.** Pieces are views keyed by the input's
  positions; per-position results are `[D]...`; per-piece results are a
  `Seq` you fold, collect or `max(by:)`.
- **Same ops everywhere.** `+`, `min`, `gcd`, `MaxSub`, `min(by:)` mean the
  same thing in `w.fold`, `windows(k).fold`, `Window`, `SegTree`, `scan`
  and `Lift`; the laws pick each structure's strategy.
- **Same order vocabulary.** Every "smaller" is `by:` with `desc` for
  larger, as in `sort`, `Heap` and `min`.
- **Same state protocol.** `Track` is accepted by `maximal`, `minimal`,
  `Sweep` and `mo`; `Window`, `Counter` and `Bag` implement it, and so can
  a user struct.
- **Costs read off the code.** Making a piece sequence is O(1); walking
  it is O(n) total for non-overlapping pieces, `windows(k).fold` is O(n),
  and re-folding each window is linted.

## 5. What goes away

| Today | Becomes |
|---|---|
| `sliding_min`, `sliding_max`, `sliding_fold` | `a.windows(k).fold(op)` |
| `MonoDeque`, `WindowFold`, `DequeFold` | `Window(op)` (round 5) |
| `Window<D>(enter, leave)` | `Sweep(state)` over a `Track` |
| `prev_less`, `prev_greater`, `next_less`, `next_greater` | `prev_smaller`/`next_smaller` with `by:` |
| `max_histogram` | `a.spans()` plus a fold |
| `max_subarray`, `max_count`, `reduce` | `a.fold(MaxSub)`, `a.fold(MaxCount)`, `a.fold(op)` |
| `diffs`, `adjacent_diff`, `with_next` | `a.pairs.map(...)`, `Diff` (round 5), `a.with_prev` |
| `record_highs` | `a.scan(max)` and a filter |
| `chunk_by`, `run_ranges` (already gone) | `runs` |
| `mo(add:, remove:)` | `mo(queries, state, answer:)`; `track(add:, remove:)` |

## Open

1. **`spans` tie-breaking.** Strict-left, non-strict-right counts each
   subarray once; a `ties:` option could expose the other three choices,
   but no dream needs them yet.
2. **`a.ring` keys.** A wrapping window's keys are a position sequence, not
   a range. The alternative is to forbid `ring.windows` and require
   `a.ring.from(i).take(k)`.
