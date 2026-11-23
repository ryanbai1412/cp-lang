# Ugly-code friction audit 1: workarounds a contestant would hate

*CP Lang 0.2.2. Read at `main` `5879196`. Report only: nothing under `crates/`, `stdlib/`, `proto/`, `site/`, `docs/` or `editors/` was changed.*

## What was read and how to read this

I read 101 ported `.cp` files from `proto/ports/` end to end (the longest ones to their first 50 to 75 lines), picked to cover every category: formatting and floats, geometry, strings, grids and simulation, greedy, DP, graphs, trees, data structures, number theory, interactive and grader tasks. The list is at the bottom. The counts quoted below ("80 lines in 62 files") are `grep` counts over all 1,001 `.cp` files under `proto/ports/`, so they include the `today/` prototype forms and are a size estimate, not an exact tally.

The question for every file was one thing: *would I be annoyed to type this in a contest?* Each entry quotes the code as it is in the repo, shows the line I wish I could write, and says where the fix belongs. There are three kinds of fix, and the difference matters:

- **Core**: a small, orthogonal change to the language itself.
- **Stdlib**: a userland helper in `stdlib/` (or the prelude).
- **Already in 0.2.2**: the cheat sheet has the short form, but ports still write the long one. Nothing to design; the fix is to migrate the ports and add a lint so the long form is never written again. These are still listed, because a reader of `proto/ports/` sees them as the language's style, and because several survive in files already migrated to 0.2.2.

Entries are ranked by how often the pattern comes up in real contests, most frequent first.

---

## 1. Printing a number with fixed decimals or leading zeros

**Fix: Core** (format specifiers), plus one **Stdlib** type.

This is the one Ryan spotted, and it is not alone. Any answer that is an exact integer scaled by a power of ten, a time, a modular answer "as four digits", or the low half of a big number needs zero padding, and 0.2.2 has `"{x:.6f}"` for floats and nothing for integers. I found eight separate hand-written versions.

`scout_batch_29/sdoi17r1p6.cp:51`:

```
fn slope(num: Int, den: Int) -> Str:
    let cut = abs(num) * 1_000_000 / den
    let units = (cut + 5) / 10
    var digits = "{units % 100_000}"
    while digits.len < 5: digits = "0" + digits
    (if num < 0 and units > 0: "-" else: "") + "{units / 100_000}." + digits
```

`scout_batch_29/wide.cp:39`:

```
var digits = "{lo}"
while digits.len < 18: digits = "0" + digits
"{hi}" + digits
```

`scout_batch_10/ctsc06p1.cp:16` and `:28` (print a residue as four digits):

```
fn digit(x: Int) -> Str = "0123456789"[x % 10..x % 10 + 1]
...
print(digit(w / 1000) + digit(w / 100) + digit(w / 10) + digit(w))
```

`scout_batch_15/sdoi17r1p4.cp:33`:

```
let scaled6 = (lo + 5) / 10
let whole = scaled6 / 1000000
var frac = scaled6 % 1000000
var digits = List.filled(6, 0)
for k in (0..6).rev:
    digits[k] = frac % 10
    frac /= 10
var out = Str(whole) + "."
for d in digits: out = out + Str(d)
print(out)
```

`scout_batch_27/noi07p1.cp:12` peels digits one at a time:

```
"{units / 1000}.{frac / 100}{frac / 10 % 10}{frac % 10}"
```

`scout_batch_35/today/coi07p1.cp:20` pads with an if-chain:

```
let pad = if frac < 10: "00" elif frac < 100: "0" else: ""
print("{th / 1000}.{pad}{frac}")
```

The same thing appears again in `scout_batch_16/ccc00s5.cp:17` (`shown`), in `scout_batch_51/today/ccc20s1.cp` (a ten-arm `digit_char` plus twenty lines building a `List<Char>`), and as a 15-line userland `fixed()` in `scout_batch_21/float.cp`.

What I want to write is a width with an optional zero fill, the same mini-language every contestant already knows from `printf`:

```
print("{w:04}")                                  // ctsc06p1: the whole helper is gone
"{hi}{lo:018}"                                   // wide.cp
print("{lo / 10_000_000}.{lo % 10_000_000:07}")  // exact integer, exact digits
```

and Ryan's function becomes one line of arithmetic and one string:

```
let units = (abs(num) * 1_000_000 / den + 5) / 10
"{if num < 0 and units > 0: "-" else: ""}{units / 100_000}.{units % 100_000:05}"
```

The specifier grammar can stay tiny: `{v:W}` (minimum width, right-aligned), `{v:0W}` (zero-filled) and the existing `{v:.Df}`. No alignment flags, no hex, no thousands separators. That is three forms, all orthogonal to the type being printed.

For the cases where the *rounding* is the hard part (round half up on an exact ratio, never print `-0.000`), `examples/local-lib/math/fixed.cp` already has a `Fixed<D>` type with `ratio(num, den)` and `Show`. Its `ratio` rounds down today; with a rounding argument, and moved into `stdlib/`, the sign-and-round logic in `slope` is written once:

```
print(Fixed<5>.ratio(num, den, round: half_up))
```

## 2. Converting an index to a number and back

**Fix: Core** (one spelling), plus a lint.

This is the most frequent source of noise in the whole folder. A position in a `domain` cannot be used in arithmetic, so every formula that mixes "where" and "how much" converts out and back in. There are four spellings of "out" across the ports (`.to_idx` 323 lines in 141 files, `.idx` 181 in 83, `.label` 126 in 71, `.ord` 52 in 30), and about 146 lines in 104 files are of the form `.to_idx + 1` or `.label - 1`.

`ccc20s2.cp:17` and `:19`:

```
for c in C: leads[Num(r.label * c.label)].push(room[r, c])
...
print(if reach[Num(rows * cols)] != none: "yes" else: "no")
```

`scout_batch_21/dpj.cp:32`:

```
print(fixed(steps[Count(dishes[Pieces(1)]), Count(dishes[Pieces(2)]), Count(dishes[Pieces(3)])], 9))
```

`scout_batch_23/noi09p5.cp:39` (every access of a 2D DP is wrapped):

```
nxt[I(i + 1), I(j + 1)] += cur
```

`scout_batch_28/noip10p2.cp:25`:

```
print(best(Left(have[0]), Left(have[1]), Left(have[2]), Left(have[3])))
```

`scout_batch_36/cco19p4.cp:80`, already in 0.2.2 syntax:

```
hulls[r].insert(Curve(cnt[r] + 1, dp[i.to_idx + 1]))
dp[i.to_idx] = hulls[r].query(cnt[r])!
```

`scout_batch_52/noi02p2.cp:11`, a 0.2.2 file, converting back to the judge's 1-based numbers by hand:

```
let ans: Int = ask("Ask", i.to_idx + 1, j.to_idx + 1, 0)
```

Two things would remove nearly all of it.

First, the language already says what to do when indices are amounts: use a structural range, `[0..=n]T` or `[1..=n]T`, whose index is a plain `Int`. Written that way the examples need no conversions at all:

```
var leads: [1..=1_000_000]List<Int> = []
for r in 1..=rows:
    for c in 1..=cols: leads[r * c].push(room[r, c])
```

```
nxt[i + 1, j + 1] += cur            // var nxt: [0..=n, 0..=n]Z
print(best(have[0], have[1], have[2], have[3]))
```

Ports reach for `domain` by habit and then pay for it on every line. A lint settles it: *a `domain` whose values are converted to `Int` in more than two places should be a range.* That is a tooling fix, not a language one.

Second, where a nominal domain is the right call, there should be exactly one way out and it should give the number the problem statement uses. 0.2.2 keeps only `.to_idx` (0-based), which is why a 1-based domain still writes `i.to_idx + 1`. The cast vocabulary already has `Int(ch)` and `Int(b)`; `Int(v)` giving the label (the value in `1..=n`) completes it, with `V(x)` as the checked way back:

```
let ans: Int = ask("Ask", Int(i), Int(j), 0)
```

Then `.label`, `.ord` and `.idx` can be deleted from the ports and the checker, and `.to_idx` stays for the rare row-major offset.

## 3. Turning a `Bool` into 0 or 1

**Fix: Already in 0.2.2** (`Int(b)`, `count`). Migrate and lint.

`(if cond: 1 else: 0)` appears on 80 lines in 62 files. It is seven tokens of pure ceremony and it usually needs its own parentheses.

`ccc17s4.cp:35`:

```
print(count(used[e] and is_new(e) for e in Pipe) - (if saved: 1 else: 0))
```

`scout_batch_28/noip05p2.cp:29`:

```
fewest[p] = min(fewest[p + hop] for hop in s..=t)! + (if stone[p]: 1 else: 0)
```

`scout_batch_23/coci21c1p2.cp:16`:

```
pref[i + 1] = pref[i] + if s[i] == 'C': 1 else: 0
```

`blind_critic_5/noiwc22p3.cp` has it four times in a row (`code = code * 2 + (if gold[i]: 1 else: 0)`).

The cheat sheet has the cast:

```
print(count(used[e] and is_new(e) for e in Pipe) - Int(saved))
fewest[p] = min(fewest[p + hop] for hop in s..=t)! + Int(stone[p])
```

The lint is mechanical: `if c: 1 else: 0` is `Int(c)`, and `if c: 0 else: 1` is `Int(not c)`.

## 4. Reading input one number at a time

**Fix: Already in 0.2.2** (`read()` of records, tuples and arrays). Migrate and lint. The 1-based shift needs a small **Stdlib** decision.

A typed `read()` is the language's best feature, and a large share of the ports do not use it. A single-scalar `let x: Int = read()` on its own line appears 1,587 times in 707 files.

`scout_batch_21/ccc14s4.cp:15`:

```
for _ in 0..n:
    let x0: Int = read()
    let y0: Int = read()
    let x1: Int = read()
    let y1: Int = read()
    let tint: Int = read()
    panes.push(Pane(x0, y0, x1, y1, tint))
```

`scout_batch_27/ioi11p1.cp:45`:

```
var ans: List<Int> = []
for _ in 0..q: ans.push(0)
var lengths: List<Int> = []
for _ in 0..q:
    let k: Int = read()
    lengths.push(k)
```

`blind_critic_5/noi04p3.cp` reads four arrays and a list of queries this way, 30 lines before any logic starts.

In 0.2.2 each of these is one line:

```
let panes: [n]Pane = read()
let lengths: [q]Int = read()
var ans: [q]Int = 0
```

Many of the long forms exist because the prototypes could not read a record or a tuple at the time. That gap is closed in the spec, so these should be rewritten, and a lint should flag "a `read()` followed by a `push` of the same name in a loop".

The related eyesore is shifting 1-based input into a 0-based domain by hand, about 60 lines in 37 files. `scout_batch_52/ctsc04p4.cp:14` (a 0.2.2 file):

```
let (u, v, w): (1..=n, 1..=n, Int) = read()
edges.push((V(u - 1), V(v - 1), w))
```

`scout_batch_39/dmpg18s5.cp:28`:

```
let (li, ri) = (l - 1, r - 1)
```

The first answer is the one the language already gives: declare `domain V = 1..=n` and read `[E](V, V, Int)` directly, so the statement's numbers are the program's numbers. When 0-based really is wanted, one sketch (`scout_batch_25/dmopc21c7p3.cp`) wishes for `read(base: 1)`. That is worth deciding one way or the other; today the cheat sheet has no such option and ports subtract by hand.

## 5. Binary search written as a `while lo < hi` loop

**Fix: Already in 0.2.2** (`bsearch_first`, `bsearch_last`). Migrate and lint.

The hand-rolled loop appears in 54 files. It is the classic place to get an off-by-one under pressure, which is exactly why the stdlib has it.

`scout_batch_26/mmcc14p3.cp:41`:

```
var (lo, hi) = (0, n)
while lo < hi:
    let mid = (lo + hi) / 2
    if outposts(mid) <= k: hi = mid
    else: lo = mid + 1
print(lo)
```

`scout_batch_21/ioi08p6.cp` has the other variant, with `(lo + hi + 1) / 2` and `hi = k - 1`, which is the one people get wrong.

```
print(bsearch_first(0..=n, r => outposts(r) <= k)!)
let side = bsearch_last(0..=min(m, n), k => cheapest(k) <= budget)!
```

## 6. The last element, and its fields, over and over

**Fix: Already in 0.2.2** (`xs.last`). Migrate and lint.

`xs[xs.len - 1]` appears on 86 lines in 61 files.

`ccc10s4.cp:26`:

```
let cost = wall[wall.len - 1].cost
```

`scout_batch_22/noi09p2.cp:27` shows the second half of the problem. Once `.last` returns an optional, a monotone-queue condition turns into a row of `!`:

```
while wins.last!.0 > i and through(i, wins.last!.0) <= through(wins.last!.1, wins.last!.0):
    wins.pop()
```

`while let` with a destructuring pattern, which 0.2.2 has, reads the way the algorithm is described:

```
let cost = wall.last!.cost
while let (from, who) = wins.last and from > i and through(i, from) <= through(who, from):
    wins.pop()
```

## 7. Casting every operand to `F64`

**Fix: Core** (a narrow change to the float rules in `decisions.md` 1.5).

Any float formula over integer input needs a cast per variable.

`scout_batch_51/ccc20s1.cp:9`:

```
let ans = max(F64(abs(b.x - a.x)) / F64(b.t - a.t) for (a, b) in obs.adjacent)!
```

`scout_batch_22/noi09p2.cp:19`:

```
fn through(j: Int, i: Int) -> F64 = best[j] + F64(line(j, i)) ** F64(p)
```

`scout_batch_36/cco19p4.cp:23`: `F64(d) * F64(d)`. And `scout_batch_50/noi04p5.cp:20` casts an exact count of tenths to a float only to print one decimal: `print("{F64(finish(0, n, m)!) / 10:.1f}")` (entry 1 fixes that one with `"{t / 10}.{t % 10}"`).

The rule that matters for safety is the other direction: an `F64` never becomes an `Int` silently, and `/` between two `Int`s never becomes float division. Both can stay. What I would loosen is only this: when one operand of `+ - * / **` is already an `F64`, an `Int` on the other side widens, exactly as an integer literal already does. Then one cast marks the expression as floating and the rest is plain:

```
let ans = max(F64(abs(b.x - a.x)) / (b.t - a.t) for (a, b) in obs.adjacent)!
fn through(j: Int, i: Int) -> F64 = best[j] + F64(line(j, i)) ** p
```

This is a change to a locked decision, so it needs a line in `decisions.md`. The cost today is real: geometry and expected-value problems are where contestants are slowest, and they are where the casts pile up.

## 8. Printing a list one per line, a count then the list, or YES/NO

**Fix: Stdlib** (one `sep:` option on `print`).

`scout_batch_27/ioi11p1.cp:71`, and 19 other files:

```
for x in ans: print(x)
```

`scout_batch_22/ccc01s5.cp:33`:

```
print(picked.len)
for p in picked: print(p)
```

`scout_batch_31/ucrpc21d.cp:23`:

```
print(sizes[0], sizes[1], sizes[2], sizes[3])
```

`scout_batch_31/coci20c6p1.cp` and nine other ports print a grid row by row (`for r in Row: print(out.row(r))`).

`plan/16-io.md` says `print` of a grid prints it by row and mentions a separate `print_sep(path, "\n")`. A second function is one more name to remember; an option on the one `print`, next to the `none:` it already takes, is not:

```
print(ans, sep: "\n")
print(picked.len); print(picked, sep: "\n")
print(sizes[..4])
print(out)
```

The YES/NO line, `print(if ok: "YES" else: "NO")`, appears on 37 lines in 31 files with every capitalisation judges use. It is short enough that I would leave it alone: a `yes:`/`no:` option would save six tokens and add a concept.

## 9. A digit to a character and back

**Fix: Core** (one operator overload on `Char`).

`s[i] - '0'` works. The way back does not, so every "print these digits" problem builds its own bridge.

`scout_batch_22/ccc99s5.cp:53`:

```
for row in rows: print(Str([Char(Int('0') + digit[Letter(c)]!) for c in row]))
```

`scout_batch_21/float.cp:25`:

```
out.push(Char('0'.ord + frac / scale))
```

`scout_batch_51/today/ccc20s1.cp:6`:

```
fn digit_char(d: Int) -> Char:
    if d == 0: return '0'
    if d == 1: return '1'
    ...
    return '9'
```

`Char - Char -> Int` is in the language. Its inverse, `Char + Int -> Char` (checked to stay in ASCII), is the missing half of the same idea, and it covers letters as well as digits:

```
for row in rows: print(Str('0' + digit[Letter(c)]! for c in row))
out.push('0' + frac / scale)
let shifted = 'a' + (c - 'a' + k) % 26
```

## 10. Re-implementing what the stdlib has

**Fix: Already in 0.2.2.** Migrate and lint.

Several ports carry a private copy of a prelude function. Each is five to ten lines that a contestant would never want to type.

Coordinate compression, `scout_batch_21/ccc14s4.cp:24`:

```
fn cuts(vals: List<Int>) -> List<Int>:
    var out: List<Int> = []
    for v in sorted(vals):
        if out.is_empty or out.last != v: out.push(v)
    out

fn rank(sorted_cuts: List<Int>, v: Int) -> Int:
    var (lo, hi) = (0, sorted_cuts.len - 1)
    while lo < hi:
        let mid = (lo + hi) / 2
        if sorted_cuts[mid] < v: lo = mid + 1 else: hi = mid
    lo
```

which is `let xs = compress(all_x)` and `xs.at(p.x0)`.

A sieve, a `gcd` and a p-adic valuation, all by hand, in `noip09p2.cp` (18 lines; `Sieve(LIM)`, `gcd` and `factor` are in the prelude). A `starts_with` in `scout_batch_22/ccc01s5.cp:15`:

```
fn starts_with(s: Str, head: Str) -> Bool = head.len <= s.len and s[0..head.len] == head
```

Counting distinct values after a sort, `ccc03s4.cp`:

```
hashes.sort()
total += 1 + count(hashes[k] != hashes[k - 1] for k in 1..hashes.len)
```

which is `total += distinct(hashes).len`. A trie that tracks its own ids, `scout_batch_22/ioi08p1.cp:32`:

```
trie.push(Node([Letter] c => none, false, false))
trie[at].kid[Letter(c)] = some(trie.len - 1)
at = trie.len - 1
```

which is `at = (trie[at].kid[c] ??= trie.push_id(Node()))`.

These are the cheapest wins in the audit: no design work, and the ports get shorter.

## 11. Comparing two optionals

**Fix: Stdlib** (`max_by`/`min_by` over optionals), no core change.

0.2.2 deliberately gives `?T` no `<`. The price shows up wherever "best so far, or nothing" is compared directly.

`scout_batch_24/cco18p5.cp:30`:

```
op: (a, b) =>
    if a == none: b
    else: if b == none: a
    else: if a!.val > b!.val or (a!.val == b!.val and a!.pos.to_idx > b!.pos.to_idx): a else: b
```

`scout_batch_36/cco19p4.cp:34`:

```
fn beats(a: ?F64, b: ?F64) -> Bool:
    if let x = a:
        if let y = b: x > y else: true
    else: false
```

`scout_batch_27/noi07p1.cp`: `if dist[s, t] == none or a + b < dist[s, t]!:`, a shape that appears on 59 lines in 45 files.

`min` and `max` already skip `none`. If `max_by` and `min_by` do the same for two arguments, the monoid is one line and the tie-break is a tuple:

```
op: (a, b) => max_by(a, b, by: p => (p.val, p.pos))
```

For the relaxation shape, `min=` already does the job (`dist[s, t] min= a + b`); where the code needs to know *whether* it improved, a `min=` that returns `Bool` (true when it changed the place) removes the `== none or ... !` pair:

```
if dist[s, t] min= a + b: ways[s, t] = ways[s, k] * ways[k, t]
```

That is a small extension of an operator the language already has, and it is the single most common reason for the `!` after a `none` test.

## 12. Testing one bit

**Fix: Stdlib** (prelude, next to `popcount`).

33 lines in 22 files, in three spellings, each relying on precedence the reader has to check.

`scout_batch_22/noi14p1.cp:23`:

```
let bit = 1 << b
if from_zeros & bit != 0: best |= bit
```

`scout_batch_25/ioi20p4.cp:39`:

```
let need = if y >> i & 1 == 1: x else: 0
```

```
if from_zeros.bit(b): best |= 1 << b
let need = if y.bit(i): x else: 0
```

`x.bit(i) -> Bool` joins `popcount`, `ctz`, `clz` and `bit_len`. It also avoids the 32-bit `1 << k` trap the prototype has.

## 13. Prefix sums in 2D with a boundary check per term

**Fix: Stdlib** (an in-place 2D accumulate beside `prefix`).

`scout_batch_21/ccc14s4.cp:60`:

```
if let up = i.prev: tint[i, j] += tint[up, j]
if let left = j.prev: tint[i, j] += tint[i, left]
if let up = i.prev:
    if let left = j.prev: tint[i, j] -= tint[up, left]
```

`scout_batch_21/ioi07p5.cp` does the same with a guard and three `!`:

```
if u.label > 1 and v.label > 0:
    seen[z, u, v] += seen[z, u.prev!, v] + seen[z, u, v.prev!] - seen[z, u.prev!, v.prev!]
```

`coci20c6p3.cp` builds Pascal's triangle with `if let a = i - 1 and let b = j - 1:`.

`prefix(a)` covers the query side (`pre.sum(l..=r)` in 1D and 2D). What is missing is the difference-array direction, where the table itself must become its running sum:

```
let tint = scan2d(diff, +)          // tint[i, j] = sum of diff over the rectangle up to (i, j)
```

One function, the 2D counterpart of the `scan(a, +)` already in the prelude. Pascal's triangle is `Comb`.

## 14. The element after this one, wrapping around

**Fix: Stdlib** (an option on `.adjacent`).

Polygons, cyclic tours and circular arrays all need "each element with the next, and the last with the first".

`btoi17p3.cp:28`:

```
for i in 0..tour.len:
    let (a, b) = (tour[i], tour[(i + 1) % tour.len])
```

`ccc10s4.cp:20`: `let (a, b) = (posts[j], posts[(j + 1) % m])`. `scout_batch_34/dmopc20c7p2.cp`: `let v = c + shape[(j + 1) % m]`.

`xs.adjacent` exists for the open case. The closed case is one flag:

```
for (a, b) in tour.adjacent(cyclic: true):
```

## 15. Packing two indices into one by arithmetic

**Fix: Already in 0.2.2** (tuples and `Bool` are indices). Migrate.

`scout_batch_27/ioi11p1.cp:33`:

```
domain State = 0..2 * n                 // 2 * fountain + (arrived by its best trail)
fn state(f: F, by_best: Bool) -> State = State(2 * f.to_idx + (if by_best: 1 else: 0))
...
let f = F(s.to_idx / 2)
let t = if s.to_idx % 2 == 1: second[f] ?? best[f]! else: best[f]!
```

`scout_batch_48/egoi22p8.cp` unpacks four 7-bit fields with `x / 128 / 128 / 128, x / 128 / 128 % 128, x / 128 % 128, x % 128`. `scout_batch_16/ioi07p2.cp` has `Side.nth(2 * w.ord + if first: 0 else: 1)`.

This is C++ thinking carried over. A tuple is an index in 0.2.2, so the encoding and both decodings disappear:

```
type State = (F, Bool)
let (f, by_best) = s
```

## 16. An array literal for a domain-indexed array

**Fix: Core** (let a list literal of the right length initialise `[D]T`).

`scout_batch_29/noip20p1.cp:15`:

```
let primes = [Small] p => [2, 3, 5][p.ord]
```

`stdlib/README.md` teaches the same shape in its first example (`[I] i => [3, 1, 4, 1, 5][i.to_idx]`). 0.2.2 accepts the literal for a range array (`var cap: [0..3]Int = [0, 0, k]` in `scout_batch_52/noi19p3.cp`), so the nominal case is the odd one out:

```
let primes: [Small]Int = [2, 3, 5]
```

The length is checked against `Small.K` at compile time when both are known, at run time otherwise. Note that on the reference interpreter this line currently means something else (it fills every cell with the whole list), which is a trap worth closing at the same time.

## 17. Giving names small integer ids

**Fix: Already in 0.2.2** (`m[k] ??= init`). Migrate.

`scout_batch_27/btoi18p1.cp:13` and, identically, `scout_batch_27/coci20c6p2.cp:12`:

```
fn id(name: Str) -> P:
    if name not in ids: ids[name] = P(ids.len)
    ids[name]
```

`scout_batch_21/nccc10s4.cp` has it inline with `contains`. With lazy insertion it is an expression:

```
fn id(name: Str) -> P = ids[name] ??= P(ids.len)
```

A related near-miss: `(if i < a.len: a[i] else: 0)` in `scout_batch_25/ioi20p4.cp:38`. `Map` has `.get(k) -> ?V`; `List` and arrays should have the same method so this is `a.get(i) ?? 0` (**Stdlib**).

## 18. Both neighbours of a position on a line

**Fix: Stdlib.**

Grid cells have `p.adj4`. A position in a 1D domain has nothing, so "for each neighbour" is a list of two optionals and an unwrap.

`dmopc21c9p4.cp:29`:

```
for nb in [u.prev, u.next]:
    if let v = nb:
        if not gone[v]:
```

```
for v in u.adj if not gone[v]:
```

`v.adj` on a 1D index (the positions one step either side that exist) is the same idea as `adj4`, one dimension down.

## 19. Looping over blocks of equal quotient

**Fix: Stdlib** (`std.math`).

The `n / k` block trick is standard in number theory, and its two index lines are easy to get wrong.

`ccc18s4.cp:11`:

```
var k = 2
while k <= w:
    let q = w / k
    let last = w / q                // the largest k with the same quotient
    total += (last - k + 1) * trees(q)
    k = last + 1
```

```
for (ks, q) in quotient_blocks(w, from: 2): total += ks.len * trees(q)
```

A generator of `(Range, quotient)` pairs; five lines of userland.

## 20. Short takes

Each of these is small, but each one made me stop reading.

**Removing from the middle of an unordered list** (**Stdlib**). `apio07p1.cp:27`:

```
free[i] = free.last()!
free.pop()
```

Wanted: `free.swap_remove(i)`.

**Sorting descending by negating** (**Already in 0.2.2**), 29 lines in 28 files. `scout_batch_31/ucrpc21d.cp:22`: `sorted(..., by: s => -s)`; `scout_batch_52/noi19p3.cp`: `by: i => -(a[i] + b[i])`. Wanted: `by: desc`, or `by: i => desc(a[i] + b[i])`. Negation is also wrong for anything that is not a number.

**A product** (**Stdlib**). `scout_batch_29/noip19p4.cp:34`: `var all = Z(1)` then `for i in Method: all = all * Z(total[i] + 1)`. `sum` exists; `product(total[i] + 1 for i in Method)` does not appear in the cheat sheet.

**A small set literal** (**Stdlib**). `scout_batch_28/dwite10c3p3.cp:10`: `let upper = Cells.empty.with(Cell(0)).with(Cell(1))`. Wanted: `Cells.of(0, 1)`.

**Manhattan distance between cells** (**Stdlib**, `std.grid`). `scout_batch_31/coci25c3p1.cp:17`: `abs(p.0 - at.0) + abs(p.1 - at.1) < k`. Wanted: `p.dist1(at) < k`.

**Points as bare tuples** (**Already in 0.2.2**, `Pt`). `cco16p2.cp:32`: `let v: Vec = (hares[h].x - p.x, hares[h].y - p.y)`, with a hand-written `cross` and `(-v.x, -v.y)`. Wanted: `let v = hares[h].at - p` and `-v`.

**A flag set in a loop** (**Already in 0.2.2**, `all`/`any`). `scout_batch_22/ioi21p4.cp:27`: `var possible = true`, a loop that sets it to false, then `if not possible`. 20 such flags in 17 files.

**Running two steps until neither changes** (**Core**, a judgement call). `scout_batch_31/sac22cc1p3.cp:38`: `while run_left() | fall(): pass`, with a comment that `or` would short-circuit. Using bitwise `|` on `Bool` to force both calls is a trick, not a statement of intent; either document it as the idiom or leave it to two lines.

---

## What I would do first

Three changes account for most of the lines above:

1. **Width and zero-fill in format specifiers** (`{n:05}`), and move `Fixed<D>` into `stdlib/`. This deletes every helper in entry 1.
2. **One index-to-number conversion, `Int(v)`**, plus the lint that points a conversion-heavy `domain` at a structural range. This is the largest single source of noise.
3. **A port migration pass with lints** for entries 3, 4, 5, 6, 10, 15 and 17, where 0.2.2 already has the short form. No design work, and it stops `proto/ports/` from teaching the long forms to the next reader.

After those, `Char + Int` (entry 9) and `Int` widening next to an `F64` (entry 7) are the two core changes with the best ratio of contest time saved to concept added.

## Files read

Formatting, floats and precision: `scout_batch_10/ctsc06p1`, `scout_batch_15/sdoi17r1p4`, `scout_batch_16/ccc00s5`, `scout_batch_21/{apio23p1, cmt21peter2, dpj, float}`, `scout_batch_22/noi09p2`, `scout_batch_27/noi07p1`, `scout_batch_29/{sdoi17r1p6, wide}`, `scout_batch_35/today/coi07p1`, `scout_batch_36/cco19p4`, `scout_batch_50/today/noi04p5`, `scout_batch_51/today/ccc20s1`.

Geometry: `cco16p2`, `scout_batch_3/cco04p4`, `scout_batch_16/ioi07p2`, `scout_batch_21/{ccc14s4, nccc10s4}`, `scout_batch_34/dmopc20c7p2`, `scout_batch_48/egoi22p8`.

Strings: `ccc03s4`, `ccc20s3`, `coci20c6p3`, `scout_batch_22/{bits, ccc01s5, ccc99s5, ioi08p1, ioi21p4, noi14p1}`.

Grids, ad hoc and simulation: `ccc20s2`, `ccc21s2`, `dmopc21c9p4`, `scout_batch_31/{cco19p2, coci20c6p1, coci25c3p1, dmopc19c4p2, noi10p3, sac22cc1p3, ucrpc21d, ucrpc21g}`, `scout_batch_52/noip20p3`.

Greedy and sweeps: `ccc21s3`, `aac4p2`, `scout_batch_21/{ioi07p5, ioi08p6}`, `scout_batch_25/{btoi19p6, dmopc21c7p3, ioi09p3, ioi16p1, ioi18p4, ioi20p3, ioi20p4, ioi22p2}`.

DP: `ccc18s4`, `ioi05p6`, `noip14p3`, `scout_batch_23/{coci21c1p2, noi09p5}`, `scout_batch_28/{dwite10c3p3, ioi04p2, noip05p2, noip08p3, noip10p2}`, `scout_batch_52/ctsc04p4`, `blind_critic_5/{noi04p3, noi08p4}`.

Graphs: `ccc10s4`, `ccc17s4`, `noip09p3`, `scout_batch_27/{acc7p3, btoi18p1, ioi11p1, noi15p1}`.

Trees: `apio07p1`, `btoi17p3`, `scout_batch_26/{btoi15p3, mmcc14p3, noip19p2, sdoi17r1p2}`.

Data structures: `noi02p1`, `scout_batch_24/{cco18p5, noi05p2}`, `scout_batch_39/dmpg18s5`, `scout_batch_52/{noi08p5, noi19p3}`, `blind_critic_5/bsspc21s4`.

Math and number theory: `noip09p2`, `scout_batch_29/{ccc19s2, noip19p4, noip20p1}`.

Interactive and grader: `yac7p5`, `scout_batch_13/btoi18p3`, `scout_batch_19/{ccc04s5, dmopc21c5p4, ioi07p1}`, `scout_batch_52/{ioi14p1, ioi17p4, noi02p2}`, `blind_critic_5/noiwc22p3`.
