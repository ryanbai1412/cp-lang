# Friction backlog, batch 29: number theory, linear algebra and contest classics

A scout pass over ten submissions where the hard part is arithmetic: a
determinant modulo a prime (shoi16p4), a Goldbach sieve (ccc19s2), exact fractions and squares past 64 bits (noip20p1,
noip19p5), gcd and lcm counting (coci25c3p5), a signed-offset modular DP
(noip19p4), a signed knapsack on a bitset (noi20p4), a product-and-max
segment tree (ioi15p4), a lazy segment tree of regression sums
(sdoi17r1p6) and exact ratio sorting (ioi09p2). The ports are in
[`proto/ports/scout_batch_29/`](../proto/ports/scout_batch_29/). Items
are numbered `O1`... so they do not collide with earlier batches.

What was checked: every port agrees with its original on 60 small random
inputs (`python3 proto/ports/scout_batch_29/check.py`, on the reference
interpreter at commit `d47d7c1`). Every "today" snippet is a line a port
contains, and every error quoted came from a probe on that interpreter.
"Wanted" snippets do not run. Nothing was run at full input size, and
nothing was run on the Rust compiler or `proto/cpc/`. The ports were
written after reading the accepted C++, not statement-first. An
eleventh port, noimps22p4, passed the same check and was dropped because
batch 23 landed with it; ccc19s2 took its place.

What the corpus no longer has: a search of the unported accepted
submissions found no Möbius or divisor-sum sieve, no CRT, and no Gaussian
elimination over GF(2) or floats. O8 is therefore a design note from
writing the modular elimination, not a finding from a GF(2) port, and
`crt`, `ext_gcd` and `mul_mod` in `std.math` were not exercised.

## The headline: the trap is right, and there is nothing to step up to

The interpreter traps `Int` overflow, and in this batch that trap was
never a nuisance: it fired exactly where the C++ needs `__int128`. But
the cheat sheet's `I128` does not exist on the interpreter, so the next
step after the trap is to write a number type by hand. Two of the ten
ports did (O1), one left the trap in (O2). After that, the arithmetic
itself was quiet: Euclidean `/` and `%` needed one workaround in ten
ports (O3) and `Mod<P>` none beyond spelling (O4). The other friction
was about containers: two silent wrong answers (O5, O6), and matrices
with no row operations (O7).

## Suggested order

1. O5 and O6, the two silent ones: `var next = ways` aliases, and
   `[D]T = [a, b, c]` fills every cell with the list.
2. O1: `I128` on the interpreter, with printing.
3. O7: `swap` of rows and `row` slices for `[R, C]T`.
4. O9: `shifted` by a negative amount.
5. The rest are spelling.

## O1. One factor past `Int`: `I128` is on the cheat sheet only (noip20p1, noip19p5)

```
error: unknown name `I128`
```

noip19p5 sums squares of block sums up to 4e16, so the answer reaches
1.6e33. noip20p1 has reduced denominators up to 60^11, about 3.6e19:
under 4 times `Int.MAX`. Both originals switch one type to `__int128`
and write a printer. The ports share a hand-written type, 53 lines with
its test:

```cp
pub struct Wide(hi: Int, lo: Int):          // hi * 1e18 + lo
    pub fn square(x: Int) -> Wide:
        let half = 1_000_000_000
        let (a, b) = (abs(x) / half, abs(x) % half)
        let cross = 2 * a * b
        let low = cross % half * half + b * b
        Wide(a * a + cross / half + low / base, low % base)
```

```cp
total = total.plus(Wide.square(s[c] - s[from[c]]))      // today
total += I128(s[c] - s[from[c]]) ** 2                   // wanted
```

Writing `Wide` was where the mistakes were possible (which products fit,
where the carry goes), and it is exactly the code a contestant should
not be writing. It did push noip20p1 to a better solution: with no wide
gcd available, the port noticed every denominator is 2^a 3^b 5^c and
keeps three exponents, so it needs only "times a digit" and "over a
digit". That is a nicer program than the original, but it was found
under duress.

Wanted: `I128` on the interpreter with `+ - * / %`, comparison,
`print`, and `I128(x)` / `Int(w)` (trapping) conversions. Printing
matters: both originals spend 15 to 25 lines on `operator<<` for
`__int128`.

## O2. Where the widening goes is a decision the language does not help with (sdoi17r1p6, ioi09p2)

sdoi17r1p6 keeps 64-bit sums in the tree and widens only the last line:

```cpp
i128 num = sxy * m - sx * sy, den = m * sx2 - sx * sx;
i128 ans6 = num * i128(1e6) / den;
```

The port does this in `Int`, and so traps on inputs the original
accepts:

```cp
print(slope(s.xy * s.len - s.x * s.y, s.len * s.xx - s.x * s.x))
```

ioi09p2 compares bills by cross-multiplying three factors,
`points * s * q`, which stays under 4e18 only because s and q are at
most 2e4; the port says so in a comment, and nothing checks the comment.
With range types read from input (`s: 1..=20_000`) the range checker in
the plan could prove both; on the interpreter `read()` does not enforce
range types, so the proof cannot be tried. This is the place where
"unproven overflow traps locally" earns its keep, and it is also the
place where a wrong guess costs a wrong answer on the judge only: the
small random inputs of `cpl stress` never reach the overflow. A stress
mode that scales values to the declared ranges while keeping n small
would catch it.

## O3. Euclidean `/` and `%`: right nine times, one workaround (sdoi17r1p6; all)

Nothing in the batch wanted truncation for its own sake. The signed
cases went the right way without thought: noip19p4's modular subtraction,
`-k - lowest` offsets in noi20p4, and `Wide.of(x)` splitting into
halves. The one workaround reproduces C++ output: the original cuts the
slope to 6 decimals with `__int128` division, which rounds towards
zero, and the slope can be negative.

```cp
let cut = abs(num) * 1_000_000 / den            // towards zero, by hand
let units = (cut + 5) / 10
(if num < 0 and units > 0: "-" else: "") + "{units / 100_000}." + digits
```

This is a formatting problem more than a division problem: the port
wanted "this exact ratio to 5 decimals" and had to build the string,
including the zero padding and the "-0.00000" case. Wanted: a fixed
formatter for an exact ratio, or `{x:.5f}` on the interpreter plus a
`trunc_div` in `std.math` beside `floor_div` and `ceil_div` for
matching C++ output.

## O4. `Mod<P>`: no arithmetic friction, some spelling (noip19p4, shoi16p4, ioi15p4)

Three ports use `Mod<P>` and none needed `ModR`, `mul_mod` or a
modulus above 2^31. What it cost was spelling:

```cp
var lap: [Rest, Rest]Z = [Rest, Rest] (i, j) => Z(0)
lap[u, v] -= Z(1)
if a[r, i] != Z(0): ...
```

`Z(0)` and `Z(1)` are on 15 lines of three ports. Wanted: integer
literals that take the type of the other operand (`lap[u, v] -= 1`,
`!= 0`), and `var lap: [Rest, Rest]Z = 0` as the cheat sheet has it.
`Prod<Z>` worked as the monoid of a segment tree with no ceremony
(ioi15p4), which is the design working.

`Mod<P>.mul` is `a.v * b.v % P`, so a modulus above 3e9 traps. No
problem here had one; with O1 fixed, `mul` can widen when `P` is large
and `mul_mod`'s doubling loop can go.

## O5. `var next = ways` aliases the array (noip19p4; silent wrong answer)

```cp
var next = ways
for d in Lead:
    if let up = d.next: next[up] += ways[d] * Z(a[i, j])
```

The first draft printed 998244330 for a sample whose answer is 3:
`next` and `ways` were one array, so each update fed the next. Batch
22 reported the same for `Str` (X4). The cheat sheet says heap values
move on last use and copy with `.copy()`; here `ways` is used after, so
this should have been a compile error asking for `.copy()`, or a copy.
The port writes `[Lead] d => ways[d]`.

## O6. `let primes: [Small]Int = [2, 3, 5]` fills every cell with the list (noip20p1; silent)

```
error: no `%` for 3 and [2, 3, 5]
```

The annotation `[D]T = value` is the fill form (`var cnt: [V]Int = 0`),
and the interpreter applied it to a list: three cells, each holding
`[2, 3, 5]`. The error surfaced two functions later. A list of the
right length is far more likely to be the contents than a fill, and the
types say so (`List<Int>` is not `Int`). The port writes:

```cp
let primes = [Small] p => [2, 3, 5][p.ord]
```

## O7. A matrix is a 2D array with no row operations (shoi16p4)

The elimination is the standard one, and the domain version reads well
where it uses ranges of the domain:

```cp
for r in i.r..:
    let f = a[r, i] * inv
    for c in i..: a[r, c] -= f * a[i, c]
```

`i..` and `i.r..` (from i, after i) remove the usual `j = i + 1`
arithmetic. Three things did not exist on the interpreter:

```
error: [0, 1, 2, ...] has no method `row`
error: unknown name `first`
```

```cp
// today: pivot search and row swap by hand
var pivot: ?Rest = none
for r in i..:
    if a[r, i] != Z(0):
        pivot = r
        break
if pivot == none: return Z(0)
for c in Rest: (a[i, c], a[p, c]) = (a[p, c], a[i, c])

// wanted (all on the cheat sheet)
let p = first(r for r in i.. if a[r, i] != 0) ?? return 0
swap(mut a.row(i), mut a.row(p))
```

The C++ swaps two `vector`s in O(1); the flat `[R, C]T` swap is O(n),
which does not change the complexity of elimination but is worth knowing.

The second point is the minor. The matrix-tree theorem wants the
Laplacian without one row and column. The original builds n by n and
`pop_back`s; a flat `[City, City]Z` cannot shrink, and a first draft
that passed "the leading k by k block" to `det` filled it with
`if r.ord < k` guards. The port declares a second domain and converts:

```cp
domain Rest = 1..n          // every city but the last
if let u = Rest.try(e.u.label): lap[u, u] += Z(1)
```

`Rest.try(e.u.label)` is honest (the last city has no row) but it is a
round trip through `Int`. Wanted: a subrange domain declared as such,
`domain Rest = City[..City.last]`, with a checked `City -> ?Rest`.

`Matrix<R, C, T>` with `det`, `rank`, `solve` and `inverse` over any
field (`Mod<P>`, `Float` with a pivot tolerance) belongs in `std`:
the 20-line `det` here is the same in every such problem, and the
original's version drops the sign of row swaps.

## O8. `Bitset` and GF(2) elimination: a design note (no port)

No unported submission eliminates over GF(2), so this was not ported.
Reading `std.bitset` against the elimination just written: `symmetric`
is the row operation and `has` is the pivot test, so a row-major
`[Row]Bitset<Col>` works today. What is missing is what O7 is missing
(swap two rows, first row with a bit set) plus one thing of its own:
`symmetric` over a suffix of columns is not needed (whole-row xor is
correct and cheap), but an xor basis for `Int` vectors (insert, reduce,
max xor) has no home. Wanted with O7: `Matrix` elimination specialised
to `Bitset` rows, and `XorBasis` in `std.math`.

## O9. `shifted` by a negative amount fails inside the library (noi20p4)

```
error: stdlib/bitset.cp:61: index 2 is out of bounds for a list of length 2
```

noi20p4's knapsack has weights `d[i] - k` of both signs; the original
writes `dp << cur` or `dp >> -cur`. `shifted(-5)` fails with an index
error from inside `bitset.cp`, which Euclidean division makes worse to
read: `-5 / 64` is -1 and `-5 % 64` is 59. The port re-bases the sum
so every step goes up, which is also less code:

```cp
let lowest = sum(min(d[i] - k, 0) for i in Item)
domain Raise = 0..=(highest - lowest)
var next = reach[i.ord].shifted(abs(d[i] - k))
next.union(reach[i.ord])
```

Wanted: `shifted(k)` for negative k, and `a | b` on bitsets as the
cheat sheet has it, so the step is `reach | reach.shifted(w)`. The
method names also differ from the sheet (`insert`/`has`/`union` against
`add`/`in`/`|`).

The domain is sized by the input (`highest - lowest`, up to 5e6 bits),
declared inside `solve()` and used as `Bitset<Raise>` in a
`List<Bitset<Raise>>`. That all worked, and `Raise.try(-k - lowest)`
turning an unreachable goal into `none` replaced a bounds check.

## O10. Non-typeable constraints: a generator in the input (noip19p5)

With n up to 4e7, noip19p5 gives a recurrence instead of the array:
six numbers, then m lines `p l r`. The port reads either form:

```cp
let (n, kind): (Int, Int) = read()
var a: [Pos]Int = [Pos] i => 0
if kind == 0:
    for i in Pos:
        let v: Int = read()
        a[i] = v
else:
    ...
```

`let a: [Pos]Int = read()` cannot sit in one branch of an `if` and be
visible after it, and `a[i] = read()` is rejected (`unknown name
read`: `read()` needs a `let` with a type), so the array is declared
filled and overwritten. Wanted: `let a: [Pos]Int = if kind == 0: read()
else: generate(...)`, with `read()` taking its type from the binding of
the whole `if`.

The generator masks to 30 bits, `(x * old + y * older + z) & ((1 << 30)
- 1)`; the products stay under 2^61, so `Int` is enough and no wrapping
`U64` was needed (unlike batch 22's X9).

Derived stress inputs cannot cover this problem: `cpl stress` would
build inputs from the `read()` types and never produce `kind == 1` with
a consistent `p` sequence, and at n = 4e7 the port keeps four arrays of
`Int` (a, s, from, queue) where the original packs `from` into 4 bytes
and frees `a`. A `[Cut]Cut` should pack to 4 bytes by its range; that
and peak RSS at 4e7 need the max-N gate, not this check.

## O11. Small things

- **`pop()` traps on an empty list** (noip20p1). The sheet has
  `pop() -> ?T`, so `while let u = ready.pop():` is the worklist loop;
  the interpreter fails with `pop from an empty list`. The ports write
  `while ready.len > 0:` and `let u = ready.pop()`.
- **`[0..=n]Int` is not an index type on the interpreter** (noip19p5),
  though `proto/cpc` takes it. The port found something better:
  prefix sums live on `Bounds<Pos>`, `s[i.r] = s[i.l] + a[i]`, with no
  off-by-one to choose. But the array then has to be written
  `[Pos.bounds] c => 0` while its type is `[Bounds<Pos>]Int`: two
  spellings of one thing.
- **A lambda is one expression** (ioi09p2). A sweep that calls back
  with the heap needed two statements in the callback, so the port
  passes `stop: ?Worker` and runs the sweep twice instead.
- **`Set` has no order on the interpreter** (ioi15p4, noi20p4): no
  `first`, `last` or `pop_first`. ioi15p4 is better without the set
  (`min_left` on a max tree); noi20p4 scans for the minimum and maximum.
- **Records in comprehensions** (noi20p4):
  `[(grams: d[i], id: i.label) for i in Item]` fails to parse
  (`expected ']' (at 'for')`); the port pushes in a loop.
- **`Sieve.factor` and `factorize`** (dropped port noimps22p4) are two names one
  letter apart for the sieve method and the trial-division function.
- **Custom `LazySegTree`** (sdoi17r1p6) took a six-field value, a
  three-field edit and three functions, and worked first time. The
  identity value needs `if t.len == 0: return t` in both `join` and
  `edit`; an `opt()` lift of a semigroup, as `std.algebra` has for
  `Monoid`, extended to actions would remove both.

## Multi-feature collisions

- **Trap + no wide type + printing** (O1, O2, O3): the three meet in
  sdoi17r1p6's last line, which needs a 128-bit product, a division
  towards zero and a 5-decimal print, and has none of them.
- **Fill syntax + list literal** (O6) and **move semantics + arrays**
  (O5): both are a rule from the sheet meeting a value kind the
  interpreter treats differently, and both give wrong values, not errors.
- **Domains + matrices** (O7): a typed square matrix cannot drop a row,
  and the fix is a second domain with an `Int` round trip.
- **Euclidean `/` + library internals** (O9): a negative shift becomes
  a negative word count and a positive bit count, and the user sees an
  index error from the library.
- **Input-sized domain + bitset + per-test function** (noi20p4): worked.
- **`Mod<P>` + `Prod` monoid + `SegTree.of` + `min_left` on a second
  tree** (ioi15p4): worked, and is shorter than the original's
  hand-written tree, `mint` and `std::set`.
