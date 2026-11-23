# Friction Backlog — Scout Batch 48 (Hard BTOI, CEOI, EGOI & DP, Part 2)

Scout Batch 48 ports ten more problems from the strict prefix partition (`dp*`, `btoi*`, `ceoi*`, `egoi*`, `apio*`, `joisc*`, `joi*`) against the locked v2 spec (`docs/one-page-cheat-sheet.md`, commit `3723dca`). The spec ports are in `proto/ports/scout_batch_48/`, the runnable ones in `proto/ports/scout_batch_48/today/`, and `check.py` in the same directory compares every runnable port with its original C++ on seeded random inputs. All ten pass.

A note on difficulty. After batches 36 and 38 the bucket has 22 unported problem codes left. Three of the ten here are genuinely hard (`btoi06p5`, `egoi22p8`, `egoi23p8`); the other seven are short, and are useful mostly for showing where everyday code still costs more lines than it should. What remains unported with a full-score original is `apio23p2`, `egoi22p3`, `egoi22p4`, `egoi22p7` and the three introductory `dpa` / `dpc` / `dpd`; the other five codes (`apio23p3`, `btoi18p5`, `ceoi22p4`, `ceoi22p6`, `egoi22p6`) have partial-score submissions only, which make poor oracles. One more small batch empties the bucket.

## Dual LOC

Lines are non-blank and non-comment, with `dbg(...)` statements dropped. "Stripped" removes the shared template header (includes, debug printer, macros), the `main` fast-IO wrapper, the hand-written lazy segment tree in `btoi06p5` (`struct Lazy` and `struct ST`, 105 lines) and the local self-test `solve()` in `egoi22p8`.

- `btoi06p5` (`4156310.cpp`): 243 raw, 109 stripped, 67 in v2 (72.4% fewer than raw, 38.5% fewer than stripped), 95 in `today/`.
- `ceoi19pp1` (`5343174.cpp`): 55 raw, 24 stripped, 10 in v2 (81.8%, 58.3%), 11 in `today/`.
- `ceoi19pp2` (`5343197.cpp`): 52 raw, 21 stripped, 13 in v2 (75.0%, 38.1%), 13 in `today/`.
- `egoi22p1` (`5198974.cpp`): 57 raw, 25 stripped, 12 in v2 (78.9%, 52.0%), 13 in `today/`.
- `egoi22p5` (`5203693.cpp`): 53 raw, 22 stripped, 16 in v2 (69.8%, 27.3%), 19 in `today/`.
- `egoi22p8` (`5202737.cpp`): 231 raw, 170 stripped, 64 in v2 (72.3%, 62.4%), 71 in `today/`.
- `egoi23p1` (`5678049.cpp`): 65 raw, 37 stripped, 20 in v2 (69.2%, 45.9%), 23 in `today/`.
- `egoi23p5` (`5678053.cpp`): 58 raw, 30 stripped, 12 in v2 (79.3%, 60.0%), 21 in `today/`.
- `egoi23p6` (`5678056.cpp`): 54 raw, 26 stripped, 11 in v2 (79.6%, 57.7%), 20 in `today/`.
- `egoi23p8` (`5659039.cpp`): 154 raw, 127 stripped, 56 in v2 (63.6%, 55.9%), 69 in `today/`.

Totals: 1,022 raw and 591 stripped C++ lines become 281 lines of v2, which is 72.5% fewer than raw and 52.5% fewer than stripped. The `today/` ports total 355 lines.

`egoi22p5` is the weakest result (27.3% against stripped) and the reason is item 48.1.

## What the originals got wrong

Differential testing found three defects in the C++, none of which the ports can express.

`egoi23p8` phase 1 loops `for (int i = 0; i < 10; i++)` over a nine-element `cmask`, reading one `int` past the vector. The read happens to return zero. In the port the loop is `for i in 0..PARTS` over a value that is produced by `digit(code, i)`, so there is no tenth element to ask for.

`egoi23p8` phase 2 computes `not_one[nbad + 1]` without checking that two candidates remain after the erasure. An early `today/` port trapped with `list index out of range` on the same input, which is how the checker found it.

`egoi23p6` allocates `long dp[m + 1][n + 1][mxs]` on the stack and segfaulted locally on a case with `n = 39`. The port's `[Take, Pos, Swaps]Int` is heap-backed.

## Remaining friction

### 48.1 — No `merge` for two sorted sequences (`egoi22p5`)

The whole problem is "subtract from a prefix, then merge two sorted halves". C++ does it in one call:

```cpp
merge(arr.begin(), arr.begin() + ind, arr.begin() + ind, arr.end(), tmp.begin(), greater<>());
```

The cheat sheet has `sort`, `sorted`, `dedup`, `rotate_left` and `reverse`, but no merge, so the v2 port spells out the two-pointer loop:

```cp
var merged: List<Int> = []
var (i, j) = (0, c)
while i < c or j < n:
    if j == n or (i < c and a[i] - x >= a[j]):
        merged.push(a[i] - x)
        i += 1
    else:
        merged.push(a[j])
        j += 1
a = merged
```

With a `merge(xs, ys, by:)` in the prelude the loop body is three lines:

```cp
for i in ..c: a[i] -= x
a = merge(a[..c], a[c..], by: x => -x)
```

This also shows a second gap: there is no short way to say "descending". `by: x => -x` works for `Int` only, and `desc(x)` is documented for heap tuples, not as a sort key. Accepting `by: desc` everywhere `by:` is accepted would close it.

### 48.2 — `List` has no positional insert (`egoi23p5`)

The solution inserts a new element at the front, the back, or between two neighbours. `List` offers `push`, `pop`, slicing and `+`, so the v2 port rebuilds the list:

```cp
path = path[..at] + [G(i)] + path[at..]
```

That is acceptable to read but allocates three lists per step, and it is not the first thing a contestant reaches for. `xs.insert(at, x)` is the obvious name. In `today/` neither slicing nor `+` on lists compiles, so the port copies element by element (21 lines against 12).

### 48.3 — A minimum that excludes one index needs a sentinel (`btoi06p5`)

The DP asks for the cheapest escape symbol other than the current run's symbol. With a `LazySegTree` the natural way is to overwrite that leaf with a huge value, fold everything, then write the real value back, which is what the C++ does with `1e9`:

```cp
const FAR = 1e18
var cost = LazySegTree<Sym, (Int, Sym)>(assign_add, (min, (FAR, Sym.last)), s => (0, s))
...
cost.apply(s..=s, (FAR, 0))
let (opt, opt_sym) = cost.fold(..)
```

The cheat sheet says "no `INF` sentinels", yet both the `min` identity for a tuple node and the masked leaf need one. Two things would remove it: let `min` / `max` over tuples take `none` as the identity so the node type is `?(Int, Sym)`, and give range structures a `fold_except(i)` (or allow `min(st.fold(..i), st.fold(i.r..))` with optional results). This is a collision between "no sentinels" (section 2) and the `(id, op)` monoid form (section 11).

### 48.4 — Custom `Action` syntax is under-specified (`btoi06p5`)

The tag here is "optionally assign, then add". The cheat sheet gives only `Action(id, compose, apply)`, without saying which argument of `compose` is the newer tag, whether `apply` receives the segment length, or how the action and the monoid are passed together. The port guesses:

```cp
type Tag = (set: ?Int, add: Int)
let assign_add = Action<(Int, Sym), Tag>(
    id: (none, 0),
    compose: (old, new) => if new.set != none: new else: (old.set, old.add + new.add),
    apply: (v, t) => ((t.set ?? v.0) + t.add, v.1))
```

One worked example in the cheat sheet would settle it. A stock `AssignAddMin` would also do, since assign-plus-add is the second most common lazy tag after plain add.

Because `proto/cpc` has only the `AssignSum` and `AddMax` stand-ins, the `today/` port could not use a lazy tree at all. Every range update in this problem covers the whole array, so it keeps a global offset, a default value, an epoch stamp per symbol and a `Set<(Int, Int)>` of the symbols written since the last assign-all. It is 28 lines longer and was the hardest part of the batch to get right.

### 48.5 — `Set` vocabulary differs between the spec and the prototype (`ceoi19pp2`, `btoi06p5`)

The spec port reads well:

```cp
good.add(x)
while let top = good.last and top > x: good.pop_last()
```

The prototype accepts `insert`, `remove`, `min()` and `max()` only, and rejects `good.len()` while accepting `good.len`:

```cp
good.insert(x)
while good.len > 0 and good.max()! > x: good.remove(good.max()!)
```

Nothing is wrong with the spec, but a contestant who learned from existing `today/` ports will type the second form. The prototype should take the spec names before more ports are written against it.

### 48.6 — Moving a `Map` bucket (`ceoi19pp1`, `egoi23p1`)

`cnt[x] += 1` on a missing key is covered by "`m[k] f= x` (monoid identity)", and it is the single biggest saving in the two short problems. The prototype compiles it as well; the `today/` ports still spell out `cnt[x] = (cnt.get(x) ?? 0) + 1`, which is one line they did not need.

In `egoi23p1` the move "take the whole bucket `u` and add it to bucket `v`" wants a `remove` that returns the old value:

```cp
if let moved = cnt.get(u - inflation):
    cnt.remove(u - inflation)
    cnt[v - inflation] += moved
```

`m.remove(k) -> ?V` would make that one line shorter and one lookup cheaper. The cheat sheet does not say what `remove` returns.

### 48.7 — No list comprehension or `collect` (`egoi23p8`)

Phase 2 splits the houses into three lists by label. The cheat sheet shows generator expressions only as call arguments (`sum(it)`, `first(it)`, `join(...)`), so building a list from one still needs a loop:

```cp
var ones: List<H> = []
var coded: List<H> = []
var flags: List<H> = []
for h in H:
    if label[h] == 1: ones.push(h)
    elif label[h] >= 11: flags.push(h)
    elif label[h] >= 2: coded.push(h)
```

If `List(it)` were allowed this would be three declarative lines:

```cp
let ones = List(h for h in H if label[h] == 1)
let coded = List(h for h in H if label[h] in 2..=10)
let flags = List(h for h in H if label[h] >= 11)
```

### 48.8 — Array literals of tuples and tuple `read` in the prototype (`egoi22p8`, several)

The seven base segments are a constant table. In the spec:

```cp
let base: [0..SEGS]Seg = [(6, 7), (7, 8), (4, 7), (4, 6), (5, 7), (0, 7), (0, 5)]
```

`proto/cpc` reports `expected (f0: Int, f1: Int), got List<(f0: Int, f1: Int)>`, so `today/` flattens the table into a `List<Int>` and indexes `ends[2 * j]`. It also rejects `let (n, m): (Int, Int) = read()` with `unknown name read`, so every `today/` port reads its header one value per line. Both are prototype gaps, not spec gaps.

### 48.9 — Two-phase and function-style problems have no runnable form (`egoi22p8`, `egoi23p8`)

`egoi22p8` is two `@grader` functions. The spec port declares them directly, but there is no way to run a `@grader` file locally, so `today/` invents a stdin protocol (`1 n` and `2 k ...`) and the checker wraps the C++ the same way. A `cpl run --grader` that reads a list of calls, or a `@grader` test block, would let the spec file itself be the runnable one.

`egoi23p8` is one program run twice with a phase number. That works today, but the only way to test it end to end is a driver that feeds the first run's output into the second. `@interactor` covers query-response judges, not this shape. Communication tasks are common in EGOI, JOI and IOI, so a co-located `@relay` block (take phase-1 output, corrupt it, produce phase-2 input) is worth considering.

### 48.10 — `@memo` with a monotone answer (`egoi23p6`)

The spec port is the best result of the batch: the recurrence is a `@memo` function and "first budget that reaches `k`" is `bsearch_first`. Two details needed thought. The table is monotone in `s` only because the recurrence is written as "at most `s` swaps", and the C++ gets the same effect from zero-initialised cells below the diagonal; the port makes the base cases explicit instead. And `@memo` over `0..=m, 0..=n, 0..cap` is 100 x 100 x 5,010 cells of `Int`, about 400 MB, where the C++ uses the same amount. The cheat sheet does not say whether `@memo` can roll a dimension when the recurrence only looks one step ahead. Here both `i` and `j` do, so the empirical DP structure detector should be able to cut this to two layers.

## Multi-feature collisions

Typed indices against packed arithmetic (`egoi22p8`, `egoi23p8`). Both problems pack small digits into one `Int` (four 7-bit quadrant masks, nine base-12 digits). The moment a value is packed it leaves its domain, and unpacking needs `.idx` going in and a constructor coming out (`Sym(e)`, `H(flags[0])`). In `today/btoi06p5` a `Set<(Int, Int)>` holds symbols as plain `Int` and converts back with `Sym(...)` on the way out. The spec allows `Set<(Int, Sym)>`, which removes the conversions.

`while let ... and ...` against `Set.last` (`ceoi19pp2`). The loop `while let top = good.last and top > x` relies on `.last` being optional on ordered sets. On `List` the cheat sheet lists `.last` next to `.first` without saying whether it is optional or traps. If the two disagree, the same line means different things on the two containers. One rule (always `?T`) should be stated.

Euclidean `%` against mirrored C++ (`egoi23p8`). The original follows its `% n` with an `if (one_sum < 0) one_sum += n` fix-up. The port writes `(known - 2 * at) % n` and is correct by definition. This is the clearest win of the division rule in the batch, and the only place where it changed the shape of the code.

## Suggested actions, in order of payoff

1. Add `merge(xs, ys, by:)` and accept `by: desc` (48.1).
2. Add `List.insert(at, x)` (48.2).
3. Put one custom `Action` example in the cheat sheet and add a stock assign-add tag (48.4).
4. Decide how a tuple `min` monoid gets its identity without a sentinel (48.3).
5. Bring the prototype's `Set` method names, tuple `read`, tuple literals and list slicing in line with the spec (48.5, 48.8).
6. Specify `m.remove(k) -> ?V` and `List(it)` (48.6, 48.7).
7. Design a runnable form for `@grader` and two-phase problems (48.9).
