# Friction Backlog — Scout Batch 44 (DMOPC 2020-2023, v2 Spec)

Batch 44 ports 10 previously unported problems from the `dmopc20*`-`dmopc23*` prefix bucket against the locked v2 spec (`docs/one-page-cheat-sheet.md`, `3723dca`). The ports live in `proto/ports/scout_batch_44/` (v2 sketches) and `proto/ports/scout_batch_44/today/` (runnable, 300/300 differential cases against the original C++ via `check.py`).

One thing to know before reading the numbers: the hard tier of this bucket was already ported. The bucket has 42 codes with an AC C/C++ submission, none from 2022 or 2023, and only 18 were unported. The 10 taken here are the hardest that remained, which means P2-P4. The reductions below are therefore flattering: short problems are dominated by template and I/O boilerplate, which is exactly what CP Lang removes.

## Dual LOC

All counts are non-blank, non-comment lines. "Stripped" keeps only `solve()` and problem-specific globals: it drops the template header, debug macros, `main()`, and generic helpers (`bpow`/`inv`/`fact`/`choose` in `dmopc21c4p4` and `dmopc21c8p2`, the difference-array `struct DS` in `dmopc21c8p2`, the `pfact` smallest-prime-factor sieve in `dmopc21c4p2`, the prime sieve in `dmopc21c5p3`). It keeps the game DP loop in `dmopc21c5p3` and the `Node` struct in `dmopc21c7p2`, since those are the solution.

Raw C++ totals **832 lines** and stripped C++ **368 lines**. The v2 ports total **128 lines**: an **84.6% reduction vs raw** and **65.2% vs stripped** (68.2% vs stripped if the 11-line `@interactor` judge in `dmopc21c2p3` is excluded, since the C++ has no judge). The runnable `today/` ports total **235 lines**: **71.8% vs raw** and **36.1% vs stripped**.

Per problem (raw -> stripped -> v2 -> runnable):

- `dmopc21c8p2`: 103 -> 41 -> 18 (-82.5% raw, -56.1% stripped) -> 34.
- `dmopc21c4p4`: 77 -> 14 -> 6 (-92.2%, -57.1%) -> 12.
- `dmopc20c5p2`: 69 -> 30 -> 9 (-87.0%, -70.0%) -> 17.
- `dmopc21c9p2`: 60 -> 25 -> 14 (-76.7%, -44.0%) -> 17.
- `dmopc21c5p3`: 122 -> 66 -> 13 (-89.3%, -80.3%) -> 31. The stripped figure includes 17 lines of dead `check()` lambda and commented-out search the author left in.
- `dmopc21c4p2`: 95 -> 24 -> 8 (-91.6%, -66.7%) -> 22.
- `dmopc21c9p3`: 86 -> 52 -> 12 (-86.0%, -76.9%) -> 29.
- `dmopc21c7p2`: 72 -> 39 -> 21 (-70.8%, -46.2%) -> 27.
- `dmopc21c2p3`: 89 -> 53 -> 23 with `@interactor`, 12 solver-only (-74.2% raw, -56.6% / -77.4% stripped) -> 32.
- `dmopc21c6p2`: 59 -> 24 -> 4 (-93.2%, -83.3%) -> 14.

The gap between v2 (128) and runnable (235) is larger than in Batch 34 (461 vs 548). Almost all of it is three missing prototype pieces: printing a list on one line, `bfs` over a `Graph`, and comprehension-level `sum`/`any`/`count` with filters. None is a language question.

## Friction in the v2 spec

### F1 — Splitting on two optionals at once (`dmopc21c8p2`)

Each column has a top and a bottom cell, either of which may be blank. The three cases (both known, one known, none known) are the whole problem. With `read(none: 0)` giving `?Int`, the natural v2 form is a tuple `match`:

```cp
let allowed = [match (top[i], bot[i]):
                   (some(a), some(b)) => (a + b)..=(a + b)
                   (some(a), none) | (none, some(a)) => (a + 1)..=(a + m)
                   (none, none) => 0..=(2 * m)
               for i in cols]
```

The sheet does not say whether an or-pattern may bind the same name in both arms, nor whether a multi-line `match` is allowed as a comprehension element. Without both, this falls back to an `if`/`elif` chain with `!` unwraps that is no shorter than the C++:

```cpp
if (!arr[i][0] && !arr[i][1]) cnt[i & 1]++;
else if (!arr[i][0] || !arr[i][1]) { int x = arr[i][0] + arr[i][1]; ds[i & 1].update(x + 1, x + m); }
else { int x = arr[i][0] + arr[i][1]; ds[i & 1].update(x, x); }
```

Note that the C++ gets the "one known" case for free from the `0` sentinel (`a + 0`); optionals are safer here but cost the or-pattern. Recommendation: state on the sheet that or-patterns may bind identical names, and show one tuple-of-optionals `match`.

The same port wants "is `x` inside every one of these ranges", written `all(x in r for r in allowed)`. That is quadratic where the C++ difference array is linear. `Steps<Int, Int>` is the listed tool, but "points covered by all k ranges" is not an obvious query on it; the runnable port hand-rolls the difference array. A `cover_count(ranges)` helper or a documented `Steps` idiom would close this.

### F2 — Peeking at the last two elements of a stack (`dmopc21c9p2`)

The merge rule is "while the top two are equal and not `z`, replace them with the next letter". v2 has `.last` but nothing for the element under it:

```cp
while pending.len >= 2 and pending[pending.len - 2] == pending.last and pending.last != 'z':
```

`pending.last` is `?Char` while `pending[len - 2]` is `Char`, so this line also compares an optional with a plain value, and `pending.first == b[matched]` in the next loop relies on the same comparison to fold the emptiness check in. The sheet should say plainly that `?T == T` is allowed and false on `none`; the port depends on it twice. A negative index or `.last(2)` slice would remove the arithmetic:

```cp
while pending.last(2) is [x, x] and x != 'z':    // proposal
```

Multi-feature collision: this problem needs a stack at the back and a queue at the front of the same sequence, so the right container is `Deque<Char>`, but `Char + 1` (next letter) and `Deque` indexing from the back are each specified in different sections and never together. The runnable port uses a `List` plus a `head` cursor.

### F3 — No linked list, and optional links fight sentinels (`dmopc21c7p2`)

Moving a block `l..r` after `k` is six pointer writes. The idiomatic v2 choice would be `next: [Node]?Node`, but then every splice needs `!`:

```cp
link(prev[l]!, next[r]!)      // noise at each of six sites
```

so the port adds head and tail sentinels to keep both arrays total:

```cp
domain Node = 0..=(n + 1)
var next: [Node]Node = [Node] v => v.next ?? v
```

This works, but it is the sentinel style the optional design is meant to retire. The domain also has to be widened by hand to hold the sentinels, and the input values `l`, `r`, `k` are read straight into `Node`, so a reader cannot tell which values are real items. A small `std.linked` (`Linked<V>` with `.move_after(l..=r, k)` and iteration) would turn the port into about 8 lines; block moves, Josephus-style deletion and "next alive" scans all recur across the corpus.

### F4 — `ask` assumes a tagged query and an explicit answer (`dmopc21c2p3`)

Same root cause as Batch 34's UD3. This protocol prints a bare number, reads a bare number, and has no final answer line: reaching distance 0 is the answer. The sheet's only example is `ask("?", x, limit: 2)`. The port writes

```cp
fn probe(guess: 0..(1 << 60)) -> Int = ask(guess, limit: 31_999)
```

and has to guess that `ask` with no tag prints just the value, that `limit:` is a global query budget rather than a per-call one, and that the `@interactor` reads the solver's line with `read_output<Int>()`. Three guesses in a 12-line solver is too many. The sheet needs one full interactive example showing both sides, a tagless query, and where the budget lives.

A smaller point from the same file: the judge wants `if a > b: a >>= 1 else: b >>= 1`. Whether compound shift assignment exists and whether a one-line `if/else` may hold statements are both unspecified.

### F5 — "k largest" (`dmopc21c6p2`)

```cp
print(sum(sorted(zero_runs, by: x => desc(x)).take(k)))
```

`desc(x)` is documented only for heap tuples, `.take(k)` is not on the sheet at all, and there is no `sorted(..., desc: true)`. Three small uncertainties for the most common post-sort operation. Recommendation: add `top(xs, k)` / `bottom(xs, k)` (partial sort in the emitted C++) and list `.take(k)` under sequences.

### F6 — Divisors from a sieve (`dmopc21c4p2`)

The sheet lists `sieve(n)` and `factor(x)`. `stdlib/math.cp` has a `divisors(x)` by trial division, which is O(sqrt x) per value and too slow at 10^6 values up to 5 * 10^6; the original enumerates divisors from the smallest-prime-factor table. The v2 port writes `sv.divisors(x)`, which does not exist, and the runnable port spends 9 lines rebuilding it from `sieve.factor(x)`:

```cp
fn divisors(x: Int) -> List<Int>:
    var ds = [1]
    for pe in sieve.factor(x):
        let base = ds.len
        var mult = 1
        for _ in 0..pe.1:
            mult *= pe.0
            for j in 0..base: ds.push(ds[j] * mult)
    ds
```

Recommendation: add `Sieve.divisors(x)` to `stdlib/math.cp` and the sheet.

### F7 — All-pairs BFS has no direct spelling (`dmopc21c9p3`)

```cp
let dist: [V][V]Int = [V] s => [V] v => bfs(g, from: [s])[v]!
```

As written this reads as one BFS per pair; it only works if the compiler hoists `bfs(g, from: [s])` out of the inner initializer, and the `!` asserts connectivity silently. The honest form needs a temporary and loses the single expression. `g.all_distances()` returning `[V][V]?Int`, or a documented guarantee about hoisting in nested domain initializers, would settle it.

## Prototype findings (today's `proto/cp.py`)

These are bugs or gaps in the runnable toolchain, not spec questions.

- **Unknown keyword arguments are silently ignored.** `sorted(xs, desc: true)` ran, returned ascending order and produced a wrong answer that only the differential harness caught. `f_sorted` reads `kw.get("by")` and drops the rest. Builtins should reject unknown keywords.
- **No `" ".join(...)` and no single-line list print.** `print(xs)` gives `[1, 2, 3]`. Both permutation-output ports (`dmopc21c5p3`, `dmopc21c7p2`) build the line by string concatenation.
- **Stdin is tokenized up front**, so interactive ports must embed their judge (already known from Batch 34).
- **Sieve speed.** `Sieve.new(5_000_000)` exceeds two minutes in the interpreter; the `dmopc21c4p2` test range is capped at 20,000.

## What worked without friction

`Mod<P>` with `Comb<P>` made `dmopc21c4p4` a single `sum` expression against 14 stripped C++ lines. `Set<(Int, Int)>` with ordered iteration replaced `set<pair<long, long>>` one for one in `dmopc20c5p2`, and Euclidean `/` and `%` needed no thought there. `runs(s)` reduced the run scan in `dmopc21c6p2` from two nested cursor loops to a filter. `@memo` over a bounded domain expressed the game table in `dmopc21c5p3` as its definition. Nominal domains caught nothing in this batch, but cost nothing either except in F3.
