# Blind critic 2: debugging a WA and pivoting algorithms mid-contest

Six corpus problems that had no port, each written the way a contest actually goes: a first
draft that is too slow or quietly wrong, a stress run to find out, then a fix or a change of
algorithm. Everything was run on the Python prototype (`proto/cp.py build`, `stress`,
`locate`) against the original accepted C++ and, where the file has one, against an in-file
`@brute(main)`. The drafts and the finals are in `proto/ports/blind_critic_2/`.

Two things up front, because they change how to read the rest.

The prototype has no `spec` block. The nearest thing is `@brute`, so that is what "spec +
gen" means below. It also has no trace, no `dbg`, and no state diff; `plan/50-debugging.md`
describes them, the prototype does not run them.

The six finals each agree with the original submission on 400 generated inputs at size 10
(`dpq` on 168, because 232 were rejected by its `where`). That is the whole of the
verification. Nothing was submitted to a judge, and the finals were not timed at the largest
input except `dpz`, by hand.

## The short version

Pivoting the data structure was the easy part. Four of the six pivots compiled on the first
or second try, and domain-typed indices caught real mistakes on the way. What cost time was
everything around the pivot: optionals that a guard two tokens away had already ruled out,
a stress tool that twice said "OK" about a wrong program, counterexamples that were not
shrunk, and no way to look inside a run.

## Problem by problem

### dpq, Flowers: `[I]Int` to `SegTree<I, Int>`

The draft is the O(n²) DP. It took three compiles, all about one optional.

```
best[i] = a[i] + max(best[j] for j in I.first..i if h[j] < h[i])        // `+` on an optional
best[i] = a[i] + max(best[j] for j in I.first..i if h[j] < h[i]) ?? 0   // the same error again
best[i] = a[i] + (max(best[j] for j in I.first..i if h[j] < h[i]) ?? 0) // compiles
```

The first message says "unwrap with `!`, `??` or `if let` first". Following it literally
gives line two, which fails with the identical message because `??` binds looser than `+`.
The message should print the parenthesised fix.

The pivot itself was one line of declaration and one of update. The only friction was the
empty prefix: "everything below `h[i]`" is `h[i] - 1`, which is a `?I`, and `query` takes
two `I`.

```
best.query(I.first, h[i] - 1)        // a SegTree<I, Int> is indexed by I, got ?I

var below = 0
if let p = h[i] - 1: below = best.query(I.first, p)
```

`let below = if let p = h[i] - 1: ... else: 0` is rejected ("expected an expression (at
'let')"), so one idea takes three lines. A half-open `best.query(..h[i])` would take one and
has no empty-range case to get wrong.

`where distinct(h)` on a `[I]I` rejects more than half of the generated inputs: the generator
does not know that a distinct `[I]I` is a permutation.

### dpz, Frog 3: O(n²) DP to convex hull trick, then a planted hull bug

Two guards the checker did not use, in one ten-line draft:

```
where all(h[i] < h[i + 1] for i in I if i != I.last)     // `i + 1` is a ?I: it may step outside I
if i != I.first:
    dp[i] = min(dp[j] + ... for j in I.first..i)         // expected Int, got ?Int
```

Both are provably fine from the guard on the same line or the line above. Both were fixed
with `!`, which is the habit this teaches: under pressure, `!` is the answer to every
optional, and then it stops meaning anything.

The hull (`List<(m: Int, b: Int)>`, a head index, a local `fn bad`) compiled first try.
Then the planted bug, both differences in `bad()` negated:

```
(r.b - p.b) * (p.m - q.m) <= (q.b - p.b) * (p.m - r.m)   // right
(r.b - p.b) * (q.m - p.m) <= (q.b - p.b) * (r.m - p.m)   // draft 2
```

```
MISMATCH size=4 seed=94 (after 693 passing)
input: 4 / 5 / 2 4 5 6      main: 20      slow: 18
```

Four numbers is small enough to trace by hand, and that is all there was to do. An `assert`
that the head line is the minimum over the kept lines did nothing twice over: stress builds
without `-DLOCAL`, so the assert is compiled out, and with it on, it passed, because the good
line had already been popped. There is no way to print `lines` per iteration without
`print`, which goes to stdout and becomes the diff.

A first planted bug (popping without regard to `head`) passed 2000 cases and turned out to be
harmless on reflection. Stress was right and I was wrong, which is worth as much as a catch.

The cross products reach about 6e18. The compiler said nothing; I checked the largest input
under UBSan by hand.

### noi15p4, Homer's Epic: a missing edge case

The draft is k-ary Huffman without padding to `(n - 1) % (k - 1) == 0`. It compiled first
try, and this was the tools at their best:

```
$ cp.py stress ...     MISMATCH seed=2    (n = 8)
$ cp.py locate ...     smallest failing input: 4 / 3 / 1 2 1 1    ours: 8 2    expected: 7 2
                       no single @brute swap fixes it; the bug is outside the tested parts
```

Four words and k = 3 makes the bug obvious: the last merge takes two. The fix is one line.
The tie-break on depth, which is the classic second bug in this problem, could not happen:
`MinHeap<(Int, Int)>` orders by weight then depth without being asked.

Two small things. `stress` and `locate` are separate commands with different behaviour (one
size and the first failing seed, against sizes ascending), and the last line of `locate` is
noise in a file with no `@brute` parts. `heap.len()` is a method while `list.len` is a field.

### dmpg16s5, Pizza Bag: O(nk) to a monotone deque, with an off-by-one

There is no deque, so the idiom is a `List` and a `head`. The first error:

```
while head < window.len and pre[window.last()] >= pre[e - 1]: window.pop()
// line 18: cannot index with ?Int
```

No column, and nothing says that `window.last()` is the optional. The planted bug was the
window bound, `<=` where `<` belongs:

```
while window[head] <= e - k: head += 1     // draft: evicts a start that is still in reach
while window[head] < e - k: head += 1
```

```
MISMATCH size=1 seed=1 (after 0 passing)
input: 1 / 1 / -1
main (exit 3): line 20: list index out of range
```

The bounds check turned a silent wrong answer into a crash with a line number on the
smallest possible input. Best diagnostic of the session, and it came from the runtime, not
from the type system.

The O(n²) warning for the `@brute` body prints on every build of the final file too. By the
third build I had stopped reading warnings.

### sac22cc2p4: grid DP to Dijkstra, and the stress run that lied

The draft assumes the best route only moves right and down. That is false, and:

```
$ cp.py stress ... 300 8     stress OK: 300 tests agree (0 rejected by where)
$ cp.py locate ...           no mismatch found (sizes 1..8, 200 seeds each)
```

1900 passing tests on a wrong algorithm. Independent random cells on a grid of at most 8 by 8
almost never force a detour. Raising the size to 12 by hand found it:

```
MISMATCH seed=105
4 12
.CCC.CCC.CCC
C..CCC...CCC
CCC.C..CC.CC
.C.C..C.CCC.
ours: 7    ref: 6
```

48 cells, not shrunk: `stress` does not shrink, and `locate` stops at size 8 with no way to
raise it from the command line. In a contest I would have submitted the draft.

The pivot was the best moment for the language. This compiled and passed first try:

```
while let (d, r, c) = pq.pop():
    if dist[r, c] != none: continue
    dist[r, c] = d
    for (dr, dc) in [(0, 1), (1, 0), (0, -1), (-1, 0)]:
        if let r2 = r + dr and let c2 = c + dc:
            if dist[r2, c2] == none: pq.push((d + cost(r2, c2), r2, c2))
```

`if let r2 = r + dr and let c2 = c + dc` makes the bounds check impossible to forget or get
wrong. The same rule was a nuisance in the draft, where "the cell above, or nothing" could
not be written as an expression.

### ccc02s4, Bridge Crossing: greedy to DP, and a tie-break bug nothing can see

Three compile round trips went on generating a name, before any algorithm. A record with a
`Str` field generates an empty string, so the first stress run reported

```
ours: line 0: unexpected end of input
```

which reads as a crash in the solution and is a hole in the generator. Then `['a' + k]` is
"expected Str, got List<Char>", then slicing a literal alphabet is "cannot index Str". What
worked was indexing a list of string literals.

The greedy draft (always send a full group) was caught and shrunk well:

```
smallest failing input: 3 / 4 / ann 1 bob 1 cy 4 dee 2
ours:     Total Time: 6 ann bob cy dee
expected: Total Time: 5 ann bob cy dee
```

Note that both lines list the same names. The output is flattened, so the grouping, which is
the answer, cannot be seen. That is the same reason the second draft passes:

```
if least == none or t <= least!:     // draft 2: keeps the last best cut
if least == none or t < least!:      // final: the first, as the original prints
```

```
$ cp.py stress ccc02s4_draft2.cp ... 500 8     stress OK: 500 tests agree
```

On `2 / 3 / ann 5 bob 5 cy 5` the final and the original print `ann` then `bob cy`; draft 2
prints `ann bob` then `cy`. Outputs are compared as whitespace-separated tokens, so the two
are equal to the harness. Whether the judge accepts either is a property of the problem that
the file has no way to state (no `checker` in the prototype), and until it can, the harness
should compare lines.

The domain fought this pivot. People are `P = 1..=n`; cut positions sit between people and
run `0..=n`. I gave up typing the positions and wrote `queue[P(j)]` and `queue[P(j + 1)]`
casts, which is where an off-by-one would live, with the checker switched off by my own
hand.

## Where the types helped and where they fought

They helped whenever an index crossed from one structure to another. `SegTree<I, Int>` keyed
by the same `I` as the array meant the `Array` to `SegTree` pivot could not mix up positions
and heights by accident. Stepping off a grid is a `?Row`, and `if let ... and let ...` is the
shortest correct way to handle it. Tuple ordering in `MinHeap` removed a tie-break bug.
Bounds-checked `List` located an off-by-one in one run.

They fought in three recurring ways. First, a guard is not evidence: `if i != I.first`,
`i != I.last` and `head < window.len` all sit next to an optional they rule out, and the
answer each time was `!`. Three of the ten compile errors in this session were this, and two more were the same optional refusing a one-line fix. Second,
`if let` is not an expression, so "the neighbour's value or a default" takes a `var` and two
statements. Third, a domain that starts at 1 has no name for the boundaries between its
elements, so prefix-style DPs fall back to `Int` and casts.

Borrow rules never came up. None of the six pivots held a reference across a mutation.

How hard were the pivots themselves? `Array` to `SegTree`: two lines, one optional. O(n²) DP
to a hull over `List<record>`: first try. O(nk) to a deque: one optional, no deque type. 2D
DP to Dijkstra over cells: first try. Greedy to DP with reconstruction: fine once the domain
was abandoned. No `Pool` pivot was attempted.

## Diagnostics that misled or wasted time

- "unwrap with `!`, `??` or `if let` first" on `a + max(...)`: appending `?? 0` reproduces
  the same error. It should print `a + (max(...) ?? 0)`.
- "cannot index with ?Int" and "expected Int, got ?Int": no column and no name for the
  sub-expression that is optional.
- "stress OK: 300 tests agree" and "no mismatch found": true, and no hint that every input
  was at most 8 by 8 or that the outputs were compared as tokens.
- "ours: line 0: unexpected end of input": the generator wrote an unreadable input; it is
  reported as the solution failing.
- "no single @brute swap fixes it; the bug is outside the tested parts": printed for files
  with no `@brute` parts.
- The O(n²) warning on a `@brute` body, on every build.

## Top five fixes, in order

1. **One stress command that shrinks, and says what it covered.** `cp.py stress file.cp`
   with no reference should use the in-file `@brute(main)` (today that needs the
   `selfstress.py` in this directory). It should run sizes ascending, shrink what it finds,
   and on success print the largest input it tried and how outputs were compared. Compare
   line by line unless the file declares a checker. This alone would have caught both
   escapes above (the grid DP and the tie-break).

2. **`dbg(...)` to stderr, and asserts on in stress.** A `dbg(lines, head)` that prints the
   line number and values to stderr and compiles out of `cpl build`, plus `-DLOCAL` in every
   stress and locate build. Then on a mismatch, rerun the smallest input and show the `dbg`
   lines of both solutions side by side.

3. **Let a guard narrow an optional.** After `if i != I.first`, `i - 1` is an `I`; after
   `i != I.last`, `i + 1` is; after `head < xs.len`, `xs.last()` is; a `min` over a range the
   guard proved non-empty is not optional. Where it cannot be proved, the error should carry
   the column, name the optional sub-expression, and print the fixed line.

4. **Half-open and boundary indices.** `tree.query(..i)`, `tree.query(i..)`, and a name for
   the `n + 1` boundaries of a domain, so prefix DPs and "everything below `h[i]`" need
   neither an optional nor a cast. And `if let` as an expression, for the default case.

5. **Generators that follow the type.** A distinct `[I]I` is a permutation; a sorted `where`
   should generate sorted; a `Str` should never be empty; grids need a structured generator
   (walls and corridors), not independent cells. And a `Deque` in the stdlib, since the
   `List` plus `head` idiom has the off-by-one built in.
