# Friction backlog, batch 28: hard dynamic programming

A scout pass over ten DP submissions: interval DPs (ioi09p4, ioi00p5),
two grid paths moved in step (noip08p3), a profile DP over subsets
(dwite10c3p3), a dense four-count memo (noip10p2), a knapsack over a running sum (noip21p3), a coordinate-shrunk
line DP (noip05p2), rolling rows (ioi04p2), a DP over the bits of a 60-bit
answer (ioi20p4) and a fee-per-photo convex-hull DP (ioi16p6). The ports
are in [`proto/ports/scout_batch_28/`](../proto/ports/scout_batch_28/).
Items are numbered `D1`... so they do not collide with earlier batches.

What was checked: every port agrees with its original on 200 random inputs
(`python3 proto/ports/scout_batch_28/check.py`, ports built to C++17 by
`proto/cp.py build` at commit `72e01c1`). Every "today" snippet is a line a
port contains, and every error quoted came from that prototype. "Wanted"
snippets do not run. Timings are from one machine, the checked local
build at `-O2`, on random inputs at the largest sizes; no adversarial
max-N input was built. Nothing was run on the Rust compiler, and
`crates/` and `proto/cpc/` were not changed. The corpus has no unported
sum-over-subsets or plain digit DP, so D4 and D6 rest on one port each.

## What read well

The interval DPs are the language at its best. Raisins is the recurrence
and nothing else, 14 lines against 78:

```cp
@memo
fn pay(r1: Row, r2: Row, c1: Col, c2: Col) -> Int:
    if r1 == r2 and c1 == c2: return 0
    var best: ?Int = none
    for r in r1..r2: best min= pay(r1, r, c1, c2) + pay(r.next!, r2, c1, c2)
    for c in c1..c2: best min= pay(r1, r2, c1, c) + pay(r1, r2, c.next!, c2)
    best! + pre.sum(r1..=r2, c1..=c2)
```

`?Int` with `min=` removed every `1e18` sentinel in the batch, and a
`?Int` return made "no such plan" a value in Post Office
(`fn best(i, k) -> ?Int`, `if let rest = best(j + 1, k - 1): ...`). The
profile DP over `Subset<Cell>` (D4) has no shift or mask in it.

## D1. `@memo` cannot be told "dense when small, hash when large"

Variance (noip21p3) has state (gaps placed, sum so far). For n up to 400
nearly every sum is reachable; for n up to 10^4 most gaps are zero and only
a sliver is. The accepted C++ carries both tables and picks by n:

```cpp
if (n <= 400) cout << dp<false>(0, 0);   // flat array, 405 * 405 * 605 / 2 slots
else          cout << dp<true>(0, 0);    // gp_hash_table
```

The port is one `@memo`:

```cp
@memo
fn rest(i: 0..n, s: 0..=top) -> Int:
    if i == n - 1: return -s * s
    ...
    min(rest(i + 1, s + tall) + n * tall * tall, rest(i + 1, s + lift) + n * (2 * d * s + lift * d))
```

The prototype sizes the table from the declared bounds (10^4 by 6 * 10^6),
sees it is over its 2^26-slot cap, and emits an `unordered_map`
for every input. Measured: 0.05 s at n = 10^4, but 1.1 to 1.4 s and 210 to
257 MB on random n = 400 inputs, where the original takes 0.15 s. A
hand-rolled table over the sum, filled in place, is the mirror image:
0.03 s at n = 400 and 2.6 s at n = 10^4. Neither single form passes the
1 s limit on both; the port ships the `@memo` and says so.

Three things would each have helped, in order of how little they add:

- Choose at run time. The slot count is known once `n` and `top` are read;
  a flat table when the actual product fits and a hash otherwise is the
  original's `if`, done by the compiler.
- Let a bound depend on an earlier parameter. With i + 1 values of at most
  `height[i]` each, the sum is at most `(i + 1) * height[i]`; the
  rectangle `0..n` by `0..=top` is at least twice the real state space,
  which is the `/ 2` in the C++.

  ```cp
  // wanted
  @memo
  fn rest(i: 0..n, s: 0..=(i + 1) * height[i]) -> Int: ...
  ```
- Report which was picked. Nothing at build time says "this memo is a hash
  map of up to 6 * 10^10 keys"; the 257 MB was the first sign.

## D2. `@memo` needs the top level, so one test case per `fn`

Packing Biscuits answers q scenarios, each with its own memo. The natural
loop does not build:

```cp
for _ in 0..q:
    ...
    @memo(hash)
    fn below(bits: 0..=TOP, lim: Int) -> Int: ...
```

> `@memo (prototype) needs a fn at top level or directly in a @multitest fn, with domain parameters`

The port became `@multitest fn solve():`, which works only because the
input happens to start with the count. Dominos Tiling has five cases and
no count; it is a bottom-up loop, so it did not matter there, but a memo
would have had nowhere to go. A memo whose table is cleared whenever the
enclosing scope is entered would cover both.

The same message says "with domain parameters", but `lim: Int` is
accepted under `@memo(hash)`. The message should say which half applies.

## D3. Rolling rows: `a = b` and `swap` fail on arrays sized by a count

Two arrays declared with the same anonymous size are not the same type:

```cp
var a: [n]Int = 0
var b: [n]Int = 0
a = b
```

> `error: line 4: expected [0..n]Int, got [0..n]Int`

and `swap(mut a, mut b)` gives `swap of [0..n]?Int and [0..n]?Int`. The
message prints the two types identically, so it cannot be acted on. The
first draft of Dominos Tiling hit it with `[0..=FULL]Int`
(`expected [range]Int, got [range]Int`); naming the dimension is the way
out, and it is the better code anyway:

```cp
var ways: [Cells]Int = 0
for _ in 0..n:
    var next: [Cells]Int = 0
    ...
    ways = next
```

That still spells the type twice and copies a row per step. Hermes avoided
a second row by updating in place, which needed its two "turn" minima
taken before the loop that shifts every entry, an ordering the reader has
to check by hand:

```cp
let turn_to_y = min(w + abs(v.ord - y.ord) for v in Coord if let w = on_x[v])!
let turn_to_x = min(w + abs(v.ord - x.ord) for v in Coord if let w = on_y[v])!
for v in Coord:
    if let w = on_x[v]: on_x[v] = w + dx
    if let w = on_y[v]: on_y[v] = w + dy
on_y[px] min= turn_to_y
on_x[py] min= turn_to_x
```

Wanted, one of: a step form that names old and new and swaps the buffers
itself, or at least `a = b` and `swap` working on same-shaped arrays with
a message that shows the difference when they are not.

```cp
// wanted
for _ in 0..n:
    ways = step ways into next: [Cells]Int = 0:
        for filled in Cells: ...
```

Adding a constant to every `?Int` entry (`if let w = on_x[v]: on_x[v] = w + dx`)
wants a lifted `on_x[v] += dx` that leaves `none` alone.

## D4. Subsets: iteration is good, building one is not

`for filled in Cells`, `Cells.full - filled`, `free.subsets` and
`[Cells]Int` all worked first time and read as the mathematics:

```cp
for filled in Cells:
    let free = Cells.full - filled
    for lying in free.subsets:
        if stands(free - lying): next[lying] = (next[lying] + ways[filled]) % MOD
```

Writing a particular subset is where it stalls. Three tries failed:

```cp
ways[Cells()] = 1                          // error: cannot call type Subset<Cell>
let none_set: Cells = []                   // error: expected Subset<Cell>, got List<None>
let pair_up: [2]Cells = [a, b]             // error: expected Subset<Cell>, got List<Subset<Cell>>
```

`Cells.empty` and `Cells.full` exist but are not on the cheat sheet, which
lists `.add`, `.with` and `.subsets` only. The two vertical placements
ended up as chains:

```cp
let upper = Cells.empty.with(Cell(0)).with(Cell(1))
```

Wanted: a set literal, `Cells{0, 1}` or `[Cell(0), Cell(1)]` coerced to
the declared `Subset`, and `.empty` / `.full` documented. No port needed
supersets or a sum-over-subsets pass, so those are untested here.

## D5. Convex hull trick: no container, no deque, no wide multiply

Aliens is the longest port (38 lines) because nothing it needs is there.

There is no line container. The cheat sheet has `SegTree`, `Fenwick`,
`SparseTable` and `Steps`, but no `LiChao` or monotone hull, and the
original spends about 75 lines on one.

There is no `Deque` in the prototype (`unknown type Deque`, though the
cheat sheet lists it), so the hull is a list and a head index:

```cp
var hull: List<Line> = []                       // live lines are hull[head..]
var head = 0
...
while hull.len - head >= 2 and useless(hull[hull.len - 2], hull[hull.len - 1], line): hull.pop()
hull.push(line)
while hull.len - head >= 2 and not (value(hull[head], x) < value(hull[head + 1], x)): head += 1
```

`hull[hull.len - 2]` twice in one line is the cost of having no
`.last(2)` or negative index.

The usual "is the middle line useless" test cross-multiplies an intercept
difference (up to 10^17 here) by a slope difference (10^6). The overflow
checker's advice is `I128`, which the prototype answers with
`unknown name I128`. The port compares the first whole x at which one line
overtakes another instead, which is exact for integer queries but took an
argument to trust:

```cp
fn overtakes(a: Line, b: Line) -> Int:
    let fall = a.slope - b.slope
    (b.at0 - a.at0 + fall - 1) / fall
```

Records order by field, so `(cost, photos)` breaks ties toward fewer
photos for free, but only `<` is emitted. `value(a, x) <= value(b, x)`
passes the front end and fails in g++:

> `error: no match for 'operator<=' (operand types are 'Tup3' and 'Tup3')`

There is no `bsearch_first` in the prototype (`unknown name bsearch_first`),
so the fee search is six lines of `low`/`mid`.

Wanted, and this is the whole inner loop of the port:

```cp
// wanted
var hull = MinLines<Plan>()                     // or LiChao<0..=m, Plan>
for i in 0..cnt:
    hull.add(slope: -2 * lo, at0: (cost: done.cost + lo * lo - shared * shared + fee, photos: done.photos + 1))
    done = hull.at(x) + (cost: x * x, photos: 0)
let fee = bsearch_first(0..=m * m, f => cover(f).photos <= k)!
```

A value type that is a record, so the photo count rides along, is the
part a plain `MinLines<Int>` would miss. No port in this batch had a
sliding-window minimum, so the monotone queue proper is untested.

## D6. Bit-by-bit DP: the state signature is fine, the arithmetic is not

The one digit-style DP, Packing Biscuits, has state (bits left, limit).
A domain and an `Int` side by side were accepted by `@memo(hash)` and the
function reads like the proof:

```cp
@memo(hash)
fn below(bits: 0..=TOP, lim: Int) -> Int:
    if lim <= 0: return 0
    if bits == 0: return 1
    let top = bits - 1                  // split on whether y has this bit
    let half = two[top]
    let stock = worth[min(top, k - 1)]
    let room = min(lim, 2 * half, stock / x + 1) - half
    below(top, min(lim, half)) + below(top, room)
```

Getting `two[top]` took four tries.

`bit_len` is on the cheat sheet and not in the prototype
(`unknown name bit_len`), which is why `bits` became a parameter at all.

`1 << TOP` with `TOP = 60` builds, and prints 0. The emitted C++ is
`(1 << TOP)` on `int`, so the shift is undefined; the port answered 0 for
every scenario with no trap in the checked build. This is a wrong-answer
bug in the prototype, not friction, and the one thing in this backlog
that should be fixed before anything else.

`2 ** top` is refused because `top = bits - 1` might be -1:
`cannot bound 2 ** top; use Mod or I128`. The line above it returns when
`bits == 0`, but the range checker does not use that. The port precomputes `let two = [2 ** i for i in 0..=TOP]`.

No port had the classic (position, tight, leading zero, sum) signature;
the corpus's three such problems were ported before. The `Bool`
parameters that shape needs are on the cheat sheet as flat-memo keys and
were not exercised here.

## D7. Constraints the types cannot say

Four inputs came with a promise that a range type cannot hold, and each
went a different way.

A bound on a combination. Biscuits promises each a[i] is at most 10^18 and
also that the whole stock, the sum of a[i] * 2^i, is at most 10^18.
Declaring the first makes the second look like an overflow:

```cp
let a: [k]0..=1e18 = read()
let worth = scan([a[i] << i for i in 0..k], +)
```

> `` `a[i] << i` may reach 5.76e+35, beyond Int; widen with I128 or tighten the input bounds ``

The port reads `[k]Int`, with no bound, so the multiply is checked at run
time instead, and states the real promise in a `where`:

```cp
let a: [k]Int = read()
let worth = scan([a[i] * two[i] for i in 0..k], +) where worth[k - 1] <= 1e18
```

So the more that is declared, the less compiles. A `where` on the read
should be able to feed the range checker, or there should be a way to say
"checked here" on one expression without dropping the type.

A relation between two inputs. Tortoise Chess: the cards sum to n - 1, and
no kind has more than 40. Both fit in `where`, and read well:

```cp
let cards: [m]1..=4 = read() where sum(cards) == n - 1
let have = [count(c == step for c in cards) for step in 1..=4] where max(have)! <= 40
```

but a generator cannot draw from them: almost no random `cards` sums to
n - 1. `check.py` builds the cards first and derives n.

A structure. Aliens' real difficulty is in spans that nest and chain;
uniform points on a 10^6 grid leave a handful of spans after the dominated
ones are dropped, and the first full-size run took 0.01 s for that reason.
The stress input that matters is "points near the diagonal", which no
range says.

A size that is only large. River's bridge is 10^9 long with 100 stones;
`let len: 1..=1_000_000_000` is exact, but the array the DP wants is
indexed by the shrunk positions, whose count is only known after the
shrinking loop. `[0..end]Bool` with `end` a run-time `let` worked without
ceremony, which is worth keeping.

## D8. Smaller things, each hit once or twice

`min` and `max` of a sequence return `?Int` even over a literal range that
cannot be empty, so three ports (ioi04p2, noip05p2, noip10p2) carry a `!`
that says nothing:

```cp
fewest[p] = min(fewest[p + hop] for hop in s..=t)! + (if stone[p]: 1 else: 0)
```

`s..=t` with `t: s..=10` is provably non-empty. The error when the `!` is
missing is clear (`` `+` on an optional (?Int, Int[0..=1]); unwrap with `!`, `??` or `if let` first ``).

A named range domain refuses arithmetic that the same range inline allows.
Kamencici (coci21c1p2, a port dropped because batch 23 landed with it)
first had `domain Held = 0..k` and failed on
`all_reds - reds.sum(l..=r) - mine` with
`'-' between Int and Held; use mine.ord for its position`. Writing the
parameter as `mine: 0..k` made it a bounded `Int`, still a dense memo key,
and the arithmetic went through. Two spellings of one thing with
different rules is a trap; the port's version is the one to teach.

Tuple patterns bind but do not assign. `var (px, py) = (Coord(0), Coord(0))`
works and `(px, py) = (x, y)` does not
(`a tuple is a value, not a place to assign to`), so Hermes ends its loop
with two lines.

`for (i, d) in gaps.items` fails with `` List<Int> has no `.items` ``; maps
have it. The port indexes.

`(if stone[p]: 1 else: 0)` appears in noip05p2, and its twin in the dropped Kamencici port. `Bool` to `Int` has no
short form.

## D9. Where features met

`@memo` and reconstruction. Post Office prints the offices, so after
`best(0, m)` the port walks the memo again, comparing optionals by hand:

```cp
while best(j + 1, k - 1) == none or serve(i, j) + best(j + 1, k - 1)! != best(i, k)!: j += 1
```

`?Int` made the DP clean and makes this line three unwraps long. A memo
that can also report the argument its `min=` chose would remove the walk.

`@memo` and `@multitest` and overflow checking (D2, D6, D7) all landed on
the 19 lines of Biscuits: the memo forced the `@multitest` shape, the
missing `bit_len` forced the extra parameter, the int-width shift gave a
silent 0, and the declared bound on `a` broke the build.

Records and ordering and emitted C++ (D5): a record as the DP value is the
idiomatic way to carry a count beside a cost, and `<=` on it is the first
thing a hull needs.

Domains and arithmetic (D8, D3): the ports are densest where indices are
positions (`r.next!`, `Row.first`) and get noisy where an index is also a
quantity. Hermes has eight `.ord` in 20 lines because a coordinate is both
a key into `[Coord]?Int` and a number to subtract.

## In priority order

1. `1 << 60` emitting a 32-bit shift (D6): silent wrong answers.
2. `<=` on records failing in g++ (D5), and `expected [0..n]Int, got [0..n]Int` (D3).
3. Cheat-sheet names the prototype lacks: `Deque`, `I128`, `bsearch_first`,
   `bit_len` (D5, D6). Each cost a rewrite of working code.
4. `@memo` choosing its table at run time and saying what it chose (D1).
5. A line container whose values can be records (D5).
6. `@memo` inside a loop body (D2); a rolling-row form (D3); subset
   literals (D4); `where` feeding the range checker (D7).
