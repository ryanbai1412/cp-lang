# Friction backlog, batch 25: greedy, ad-hoc, bitwise and constructive

A Stage-1 scout pass over ten submissions: a 64-bit Gray code, a three-key
ranking, a sorted-window greedy, a bit-reversal construction, top-bit
recursion, an exchange-argument greedy, fracturing search over a heap, bitset
subset sum, a constructive strategy table, and an XOR step on a graph. The
ports are in
[`proto/ports/scout_batch_25/`](../proto/ports/scout_batch_25/): noip19p1,
ioi09p3, ioi16p1, ioi18p4, ioi20p4, ioi20p3, btoi19p6, noi20p4, ioi22p2 and
dmopc21c7p3. Items are numbered `GB1`... so they do not collide with earlier
batches.

What was checked: the ports were written against commit `d2d9933`. All ten
`today/` ports build with `proto/cpc` at that commit and pass
`proto/ports/scout_batch_25/verify.py` under ASan and UBSan against their
original C++. Every error message quoted was produced by `proto/cp.py build`
at `d2d9933`, followed by `g++ -std=c++17 -O2 -fsanitize=undefined` where the
message is a C++ or runtime one. The "wanted" snippets do not build. Nothing
was run at the full limits.

Two limits on this batch. The corpus has no unported XOR linear basis or
meet-in-the-middle submission (text search only), so those themes are absent
and the `Subset` domain was not exercised. And four problems first picked
(noi14p1, noi21p4, btoi19p2, btoi19p4) were landed by batches 20 and 22
while this batch was being written. Their drafts built and passed the same
kind of differential test before they were removed, and three of them are
where the bitwise friction was first met. A snippet marked "dropped draft"
below comes from one of those four and is not in the repository; each one
is backed by a small standalone program that was rebuilt to confirm the
message.

## Suggested order for a Stage-2 sprint

1. GB1: `1 << e` is emitted as a 32-bit C++ shift. It is undefined behaviour
   in the emitted C++ and a wrong answer without a message, so it breaks the
   zero-UB goal. It is the only item here that is a trap and not a missing
   convenience.
2. GB2: no unsigned or wrapping 64-bit integer.
3. GB3 and GB4: `popcount`, `~`, `<<=`, hex literals, `^` on `Bool`. Small,
   and every bitwise port wanted at least one.
4. GB5: `Bitset` in the compiler.
5. GB6 and GB7: `desc(x)` and a heap ordered by a field.
6. The rest in any order.

## GB1. `1 << e` is a 32-bit shift in the emitted C++ (bug)

When the left operand is a literal and the compiler knows a range for the
shift amount, the emitted C++ is a plain `1 << e` on `int`. This is the
smallest case:

```cp
let a: 0..=40 = read()
print(1 << a)
```

With input `40` it prints `256`, and UBSan reports
`shift exponent 40 is too large for 32-bit type 'int'`. A constant is hit
the same way: `let M32 = (1 << 32) - 1` is emitted as
`M32 = (((1 << 32)) - 1);`. A loop index too, which is how ioi20p4 met it:

```cp
for i in 0..60:
    var cap = min(1 << (i + 1), s[i] / x + 1) - (1 << i)     // ioi20p4, before
```

The same file had `1 << j` with `j` computed at run time, and that one was
emitted as the checked `cp_shl(1, j, line)`. So the wrong path is the one
the range checker believes it has proved safe. `1 << 31` is in the same
group: the dropped btoi19p2 draft packed 32-bit words with `1 << (c % 32)`
and got a negative word (`left shift of negative value` one line later).

The workarounds, none of which a contestant would guess:

```cp
let M32 = 4294967295                      // noip19p1: the literal
fn bit(i: Int) -> Int: 1 << i             // ioi20p4: a parameter has no range, so it is checked
```

Wanted: `<<` always computes in `Int` (emit `1LL`, or `cp_shl` when the
amount is not proved to be in `0..=62`), and the differential fuzzer gets a
case with a literal left operand and a ranged right operand.

## GB2. No unsigned 64-bit, no wrapping arithmetic

noip19p1 is three lines when `k` has a type:

```cp
let (n, k): (1..=64, U64) = read()
let g = k ^ (k >> 1)
print(join(g.bit(i) for i in (0..n).rev))
```

`U64` is `unknown type U64` today, and `k` goes up to 2^64 - 1, so it cannot
be read as `Int`. The port reads it as a digit string and multiplies by ten
across two 32-bit halves, then takes bits from whichever half holds them:

```cp
let digits: [domain Dg]Char = read()
for i in Dg:
    let lo = kl * 10 + (digits[i] - '0')
    kh = (kh * 10 + (lo >> 32)) & M32
    kl = lo & M32

fn bit(i: Int) -> Int:
    if i >= 64:
        return 0
    if i >= 32:
        return (kh >> (i - 32)) & 1
    (kl >> i) & 1
```

That is 22 lines against 17 lines of C++. The dropped noi21p4 draft was the
worse case: the statement gives a xorshift generator, `k3 ^= k3 << 23` and
so on, which must wrap. On a checked `Int` it traps, so the state was
carried as halves and every shift was written twice:

```cp
let th = k3h ^ (((k3h << 23) | (k3l >> 9)) & M32)          // dropped draft
let tl = k3l ^ ((k3l << 23) & M32)
```

A related gap: `let a: 0..=18446744073709551615 = read()` is accepted
without a message although the range does not fit `Int`.

Wanted: a `U64` whose `+`, `*` and `<<` wrap, whose `>>` is logical, and
which `read()` and `print` handle. Statement-given generators and hashing
are the uses. It should not mix with `Int` without a conversion.

## GB3. `popcount`, `bit_len`, `ctz`, `bitrev` are missing in the compiler

`popcount(m)` is `unknown name popcount` in `proto/cpc` (the interpreter has
`popcount`, `ctz`, `bit_width` and `bit_ceil`). The ports wrote the bit
loops out:

```cp
fn top_bit(v: Int) -> Int:                      // ioi20p4: __lg
    var i = 0
    while (v >> (i + 1)) > 0:
        i += 1
    i

fn bitrev(p: Int) -> Int:                       // ioi18p4: reverse the low L bits
    var r = 0
    var v = p
    for b in 0..L:
        r = r * 2 + (v & 1)
        v = v >> 1
    r

var L = 1                                       // ioi18p4: bit_ceil
var leaves = 2
while leaves < n + 1:
    L += 1
    leaves = leaves * 2
```

Wanted: the four interpreter names in the compiler, lowered to the builtins.
`bitrev(p, L)` is rarer and can stay userland. The two tools also disagree
on a name: the ideal ports wrote `bit_len` from the cheat sheet and the
interpreter calls it `bit_width`. Pick one.

## GB4. Bitwise operators: what works and what does not

Precedence is right, and it is worth saying so because it is the C trap:
`a & 4 != 0` parses as `(a & 4) != 0`, and `a | 2 <= 7` as `(a | 2) <= 7`.
The ports were first written with defensive parentheses, and they are not
needed.

Four things do not work:

```cp
print(~a)              // error: expected an expression (at '~')
v <<= 2                // error: expected an expression (at '=')      (&=, |=, ^= are fine)
print(a ^ 0xFFFF)      // error: unknown name xFFFF
print(b ^ c)           // error: `^` needs Int operands, got Bool and Bool
```

dmopc21c7p3 reads its lamps as `[V]0..=1` instead of `[V]Bool` only so that
`after[v] ^ lit[u]` typechecks, and then compares `lit[s] == 1` everywhere.
The `0xFFFF` message is the worst of the four: the lexer reads `0` and then
a name.

Wanted: hex and binary literals; `<<=` and `>>=`; `^` on `Bool` (or a
message that says to write `!=`); and for `~`, either the operator or a
message that names the replacement, since on a checked signed `Int` the
useful form is "complement within k bits".

## GB5. `Bitset` exists in `stdlib/` but not in the compiler

noi20p4 needs a subset sum over offsets `d[i] - k`, which can be negative,
and one bitset per prefix to walk back through:

```cp
domain Off = -n * k..=n * k
var reach: [0..=n]Bitset<Off> = Bitset()
reach[i.ord + 1] = reach[i.ord] | reach[i.ord].shifted(d[i] - k)
```

The port keeps one `Bool` per cell in a flat `List<Bool>` with the offset
shifted by hand, and is not sized for the full limits:

```cp
for o in 0..W:
    var r = reach[i * W + o]
    if o - c >= 0 and o - c < W and reach[i * W + o - c]:
        r = true
    reach.push(r)
```

The dropped btoi19p2 draft packed words by hand, and the word was 30 bits,
not 64: with a checked signed `Int`, `here << 1` on a full word traps, and
32 bits ran into GB1.

Three things to settle when `Bitset` lands in the compiler: a shift by a
negative amount (the original C++ branches on the sign and uses `<<` or
`>>`); a bitset over a domain that does not start at 0; and `|`, `&`, `|=`,
`&=` between bitsets, which `stdlib/bitset.cp` spells `union` and
`intersect`.

The same problem shows a limit no type states: the original holds `n + 1`
bitsets of several million bits as a local array and needs an unlimited
stack to run locally (`verify.py` raises it for the originals).
`[0..=n]Bitset<Off>` must go on the heap however it is declared.

## GB6. `desc(x)` does not exist; the key is negated (3 ports, 4 sorts)

`sort(mut xs, by: p => desc(p.0))` is `unknown name desc`. ioi09p3 is the
plain case, a ranking on three keys, two of them descending:

```cp
let ranking = sorted(C, by: c => (desc(score[c]), desc(tasks[c]), c))    // wanted
print(score[C(P)], ranking.position(C(P)) + 1)
```

Today the keys are negated into a tuple, and the answer is un-negated on the
way out:

```cp
ranking.push((-score, -tasks, c.to_idx + 1))
sort(mut ranking)
...
print(-ranking[r].0, r + 1)
```

The original C++ does exactly this (`sarr[i] = {-cs, -cnt, i}`), so the
port is no better than the C++ here. The negation leaks further in ioi20p3,
where `prize -= u.0` means "add the gain", and in btoi19p6, where the heap
holds `(-value, id)` and the answer is `-top.0`. Negation also has no
equivalent for a `Str` or tuple key.

A second pattern in the same three ports: sorting indices by a key. With no
`sorted(I, by: i => w[i])` over a domain, ioi16p1 builds `(weight, index)`
pairs, sorts them, and reads `.0` and `.1`:

```cp
var order: List<(Int, Int)> = []          // (weight, index), lightest first
for i in I:
    order.push((w[i], i.to_idx))
sort(mut order)
```

ioi20p3 is only correct if ties in the gain keep `j` ascending within a
colour. The port gets that from the tuple `(-gain, c, j)`; the wanted form
`by: (c, j) => desc(gain(c, j))` gets it only if `sorted` is stable. Say in
the docs that it is.

## GB7. A heap of records ordered by one field (1 port)

btoi19p6 pops subproblems by value:

```cp
struct Sub(value: Int, team: List<P>, forced: Int, banned: Set<P>)
var heap = MaxHeap<Sub>(by: .value)
```

The compiler's heap orders tuples of `Int`, so the port keeps the payload in
three parallel flat lists and pushes `(-value, id)`:

```cp
var teams: List<Int> = []        // K members per subproblem
var bans: List<Bool> = []        // N flags per subproblem
var forced: List<Int> = []
heap.push((-value, count))
```

Every access is then index arithmetic, `teams[src * K + i]`,
`bans[base + p]`, and a failed subproblem has to pop its half-written rows
to keep the lists aligned. That is 61 lines against 20. Also
`MinHeap<(Int, Int)>` has no `.len` (`has no .len`), so "while the heap is
not empty" cannot be written.

## GB8. Tuple unpacking

What works today: `let (p, q) = xs[0]` and `for (u, w) in xs:`. What the
ports could not write:

```cp
let (n, m): (1..=1e5, 0..=1e9) = read()              // error: unknown name read
var (x, ans) = (0, 0)                                 // builds, then `x += 1` is: `x` is not a var
```

The second one is the trap: the declaration is accepted and the names are
bound as immutable, so the error appears at the first assignment and says
the opposite of what was written. Every port therefore has one `let ... =
read()` per input value and one `var` per line; ioi20p3 opens with three
reads for `n m k`.

Returning several lists has the same shape. ioi18p4 returns
`(C: List<Int>, X: List<Int>, Y: List<Int>)` in the ideal port; the
`today/` port keeps `X` and `Y` as top-level `var`s that the recursive
`build` appends to.

One message was good and is worth keeping. Passing an element of a list to
a function that also changes that list (btoi19p6):

```
`teams[((id * K) + i)]` can't be passed to `spawn`: `spawn` also changes `teams` directly, so it would reach the same storage under two names
  help: pass everything `spawn` needs from `teams` as parameters, or copy the value out first
```

`let out = teams[id * K + i]` fixed it on the first try. The value is an
`Int`, though, and an `Int` argument is a copy, so the rule could simply not
fire for by-value scalars.

## GB9. Bare-core ad-hoc loops

**Counting down.** `for i in (0..n).rev:` builds and is what noip19p1 uses.
noi20p4 was written before that was checked and still reverses by hand
(`let i = n - 1 - back`). The cheat sheet shows `.rev` only on a domain,
which is why the range form was not the first thing reached for.

**A domain per test case.** `domain I = 0..n` and a typed `read()` inside a
`fn` build today, so a per-case function can keep typed indices. noi20p4
does not use this: it was written with `Int` indices and `List`s before the
check was made, and its helper `greedy` takes groups of ingredients from
either the whole case or one half of a split, which is easier with plain
lists. The ideal port wants `@cases fn case():`.

**Index conversions.** Arithmetic on a typed index is noisy. In ioi20p3 the
tickets of a colour are taken from both ends:

```cp
ups.push((-(x[c, Ticket(m - 1 - j)] + x[c, Ticket(k - 1 - j)]), c.to_idx, j))
...
let c = Colour(by_plus[rank].1)
round[c, Ticket(hi[c])] = r
```

An index goes into a tuple as `c.to_idx` and comes out as `Colour(...)`
because the sortable tuple holds `Int`s only. `Ticket.last - j` and tuples
that can hold a typed index would remove all five conversions.

**A name from `<complex>`.** A top-level `var arg: [D]Int` builds, and then
g++ fails with `reference to 'arg' is ambiguous` against `std::arg` (met in
the dropped noi14p1 draft). The emitted C++ should mangle user globals or
not pull `std` names into scope. The user sees a C++ error for a legal CP
Lang program.

**Grader-style tasks.** Five of the ten are IOI functions (`find_subset`,
`create_circuit`, `count_tastiness`, `find_maximum`, `devise_strategy`). The
`today/` ports read a made-up stdin format, and `verify.py` appends a `main`
to the original. A `@grader fn` that `cpl build` can emit as the bare
function is the missing piece; batch 19 adapted its graders to stdio the
same way.

**Returning a table.** ioi22p2 returns `List<List<Int>>` where the cell is
really "A is smaller, B is smaller, or go to state s". The ideal port still
encodes that as `-1`, `-2` and a state number because the grader's interface
does. An enum on the inside with a conversion at the boundary would keep
the two verdicts from being swapped by a sign slip.

## GB10. Constraints that cannot be typed

**Sums across the input.** ioi20p4: "the total tastiness in the pantry is at
most 1e18". The port types `a: List<0..=1e18>` and `x: 1..=1e18`, which
says nothing about `sum(a[i] << i)`. The running sum `run += a[i] << i` is
safe only because of the untyped sentence, and a random generator that
respects the per-element range overflows at once (the generator in
`verify.py` spends a budget as it goes). Wanted:
`assume sum(a[i] << i for i in 0..a.len) <= 1e18` that both the overflow
checker and the generator use.

**A promise the algorithm depends on.** ioi16p1 is correct only because
`u - l >= max(w) - min(w)`. A hand-typed input that breaks the promise
(`l = 10, u = 12`, weights `3 8 5 1`) makes the port print an empty subset
although `3 + 8` fits. The `read()` types accept that input, so nothing
says the answer is outside the contract. Wanted: an `assume` on the input
that `cpl run` checks in debug mode and the generator obeys.

**Shape coupling.** ioi20p3: `n` even, `k <= m`, every row sorted. noi20p4:
`sum(d) == m * k` and `m >= n - 2`. noip19p1: `k < 2^n`, where `2^n` itself
is not an `Int` at `n = 64`. None of these is a range. The generators in
`verify.py` are 4 to 12 lines each for this reason (sorted rows; `n`
positive parts of `m * k`; a budget of tastiness), and each one is logic a
`gen` block would have to hold.

**A wider type on the other side.** The ioi16p1 grader takes `l` and `u` as
`int`. The first generator produced `u` above 2^31 and the original
overflowed while the port did not. The port's `Int` hid a limit that the
statement has and the `read()` type should carry.

**Stress size is not the judge size.** noi20p4's `m = n - 2` branch is
reached by a small share of random cases unless `m` is forced, and ioi22p2
is only interesting at `N = 5000`, where trying every pair of bags is 25
million simulations. A `gen` block needs a way to say "bias towards this
branch", and `cpl stress` should report which branches of the solution the
cases reached.

**Brute forces with their own limits.** The ideal ports give each brute a
`where` clause (`where n <= 12`, `where a.len <= 6 and sum(a) <= 40`,
`where N <= 60`). In two of the three the limit is on a derived quantity,
not on an input variable, so the generator has to solve for inputs that
satisfy it.

## What read well

The ideal ports are 245 lines against 580 lines of C++ with the template
removed. The two that came out closest to the statement were ioi09p3 (9
lines; the ranking is one `sorted` call) and ioi16p1 (the window is 8
lines and returns `order[lo..=hi]`). The `&` precedence and the aliasing
message in GB8 are the two places where the language as built today was
better than C++.
