# 1.2 Values, types and representation

Status: decided unless marked *proposed* or *open*.

Two promises run through this doc. First, the obvious type is the right
one: `Int` never overflows on contest numbers, `?T` replaces every
sentinel, and records replace `pair<pair<int,int>,int>`. Second, every
value takes the bytes a careful C++ programmer would give it, and nothing
changes representation behind your back (G2, G9).

## Integers

### `Int`

`Int` is 64-bit signed, and it is the default for every number. A C++
habit this removes, from the user's own rejected-then-accepted diffs:

```
// C++: RTE, then AC after changing int to long
int x;                      // sac22cc4p1
int acnt[26] {};            // mccc3j3

// here: nothing to remember
let x: Int = read()
```

`Int.MIN` is `-(2^63 - 1)`, not `-2^63`. The one value below it is
reserved as the niche for `?Int`, so `?Int` is 8 bytes. In return,
`-Int.MIN` and `abs(x)` can never overflow. Code that genuinely needs the
full 64-bit pattern uses `U64` or `I64`.

### Range types

A range type is an integer with a declared range: `0..=1e9`, `1..=n`,
`0..26`. The range is checked where a value enters the type, proven by the
compiler where it can be, and used for three things.

**Storage width.** Arrays and fields of a range type use the smallest width
that fits, so declaring the constraint also saves memory:

```
let a: [V]0..=1e9 = read()      // 4 bytes per element
let c: [V]0..26 = ...           // 1 byte per element
let d: [V]?0..=1e9 = none       // still 4 bytes: the spare values hold none
```

Arithmetic always happens in 64 bits: `a[u] + a[v]` is an `Int` and never
wraps at 32 bits. Only stores narrow, and the compiler proves each store
fits or inserts a check (debug runs) and reports it (`cp build --checks`).

**Overflow analysis.** Ranges propagate through arithmetic (doc 3.5), so
`a[i] * a[j]` is known to be in `0..=1e18` and a sum of 2e5 of them is
known to overflow. That is a compile error with the bound, not a WA.

**Documentation.** `n: 1..=2e5` says what the problem says. Input reading
checks it in debug runs, generators use it (doc 1.9), and cost warnings use
it.

Range bounds can be runtime values (`1..=n`), in which case the width
comes from the declared type of `n`.

### Fixed-width types

`I32`, `I64`, `U8`, `U32`, `U64` and `I128` are for bit-level work and
exact C++ interop. `U64` wraps on overflow and is where bit operations
are most natural (`popcount`, shifts, masks; *proposed*, decision 1.8).
`I128` exists only where the judge profile supports `__int128`, and is
only used where you write it, so performance stays predictable.

Conversions between integer types are explicit: `x.as<I32>()` checks the
range, `x.wrap<U32>()` truncates.

### Division and remainder

`/` rounds toward negative infinity and `%` is never negative (decision
24: "we should just get rid of the negative remainder bug"):

```
-7 / 2   // -4
-7 % 3   // 2
(i - 1) % n   // always a valid index into 0..n
```

When both operands are provably non-negative, which is nearly always,
these compile to plain C++ `/` and `%` with no extra instructions.

### Literals

`1e9`, `2e5` and `1e18` are integer literals when they are whole numbers.
`1_000_000` uses underscores. `0b1011` and `0xff` are binary and hex.
`2^20` is a power in constant expressions; xor is `xor` (decision 23).

### Overflow

- Proven impossible: no check.
- Provably possible: a compile error naming the expression and the input
  bounds responsible, with the fix (`I128`, a tighter bound, or `assert`).
- Unknown: trapped in debug runs, unchecked in the judge build
  (*proposed*, decision 7.2).

## Floats

`F64` is the only float type. `==` on floats is a lint (*proposed*,
decision 1.5) that suggests `abs(a - b) < eps`. Printing uses a fixed
precision chosen per call (`print(x, digits: 9)`), never C++'s default of
six significant digits.

## Booleans, characters, strings

- `Bool` is 1 byte. `[V]Bool` is a byte array; `BitSet<V>` is the explicit
  packed form, with word-level operations.
- `Char` is a byte, with literals `'a'` and arithmetic as an integer
  (`c - 'a'` is a `0..26` when `c` is proven lowercase).
- `Str` is a byte array with its own domain, so every sequence tool works
  on strings: `runs(s)`, `windows(s.idx, k)`, `s[r]` for a slice
  (*proposed*, decision 1.6).
- String literals interpolate: `"{a} {b}"`, with `{{` for a brace. Raw
  strings are `r"..."`. The user asked whether interpolating everything is
  "a bit weird" and accepted it: it removes `f"..."` noise from every
  print.

## Optionals

`?T` is the only way to express absence. There is no `-1`, no `INF`
meaning "unreached", no `nullptr`. The C++ habit and its replacement:

```
// C++
vector<long long> dist(n + 1, LLONG_MAX);      // and later: if (dist[v] == LLONG_MAX)
int best = -1;                                 // and -1 is also a valid answer?

// here
var dist: [V]?Int = none
var best: ?V = none
```

### Representation

An optional uses a niche whenever `T` has a spare value, so it costs no
extra bytes:

- domain positions and handles: `?V` is 4 bytes, using `u32::MAX`;
- range types with spare values: `?0..=1e9` is 4 bytes;
- `Int`: `?Int` is 8 bytes, using `-2^63`;
- enums with unused tags, and records whose first field has a niche.

Otherwise it is `T` plus a flag byte, laid out with the record. Hover
always shows which (G9), and `sizeof` checks on core types are part of
validation (doc 8.2).

### Using optionals

```
let d = dist[v] ?? -1                  // default
let u = parent[v]?                     // early exit from the fn (or do: block)
if let d = dist[v]: print(d)           // bind when present
match best:
    none => print(-1)
    some(v) => print(v)
dist[v] = min(dist[v], nd)             // none counts as +infinity in min
```

`min`, `max` and `max=` treat `none` as the identity (`min(none, x)` is `x`),
which is what every shortest-path and DP update wants. Comparing an
optional with `<` is an error, so `none` never silently sorts first.

## Records, structs and enums

### Anonymous records

For one-off shapes, especially return values (the user: "i really want
anonymous records for one off shapes"):

```
fn span(v: V) -> (lo: Int, hi: Int):
    ...
    (lo: a, hi: b)

let (lo, hi) = span(v)
let s = span(v); print(s.hi - s.lo)
```

Two records with the same field names and types are the same type.
Arrays of records are stored as arrays of structs, fields in declaration
order (*open*: an explicit `@soa` for struct-of-arrays).

### Structs

Nominal, with fields, defaults and methods in one body:

```
struct Edge:
    u: V
    v: V
    w: 1..=1e9 = 1

    fn other(self, x: V) -> V = if x == self.u then self.v else self.u
    @get fn heavy(self) -> Bool = self.w > 1e6
```

- Construction uses field names: `Edge(u: a, v: b, w: 5)`; positional
  construction is allowed when the struct has at most three fields.
- `e with w: 7` makes an updated copy.
- Methods take `self` (borrowed) or `mut self`; call sites don't mark
  `mut` on receivers (decision 15).
- Properties without parentheses need `@get` (the user: "it feels like it
  should be an explicit @ annotation").
- Copying a large struct needs `.copy()`; moving at last use is free.

### Enums

Enums have payloads, are matched exhaustively, and come in two forms
(decision 18):

```
enum Dir = Up | Down | Left | Right

enum Query:
    Add(v: V, x: Int)
    Ask(l: V, r: V)
    Undo

for q in queries:
    match q:
        Add(v, x) => seg.add(v, x)
        Ask(l, r) => print(seg.fold(l..=r))
        Undo => history.undo()
```

The tag uses the smallest width that fits, or a niche in a payload when
one exists. A capitalized name in a pattern must be an existing variant,
so a misspelled variant is an error rather than a catch-all binding. The
user doesn't use C++ enums "cuz c++ enums suck"; ports should reach for
enums where a problem has kinds of queries or states.

## Representation traits

These are known to the compiler and derived automatically when the
representation allows:

- `Niche`: has a spare bit pattern for `?T`.
- `Index`: maps densely to `0..k` (domains, small enums, `Bool`).
- `Copy`: cheap to copy (at most 16 bytes, no owned heap data).
- `Relocate`: safe to move with `memcpy`.
- `Zero`: all-zero bytes is a valid value, so arrays can be zero-filled.
- `Bits`: a fixed bit width, for packing into `BitSet` and masks.
- `Small`: fits in a register.
- `OrdKey`: maps to an integer preserving order, for radix sorts and
  coordinate compression.
- `Bounded`: has `MIN` and `MAX`.

They let stdlib code (and user code) choose a representation, but only
when the choice has no trade-off. Dense forms such as `BitSet<V>` and
`[K]?V` are always explicit types; a `Set<V>` stays an ordered set even
when `V` is small (the user: "we should be very careful with things we do
secretly under the hood").

## What's never implicit

- A large copy.
- A change of container representation: a set never silently becomes a
  bitset, nor a map a slot array.
- Bit-packing of fields.
- A sentinel value that the program could also store.
- An indirect call: a fn chosen at run time needs an explicit `dyn`.
- A heap allocation the source doesn't show.

Every implicit choice that is allowed (a niche, a narrowed field, a shared
instantiation) shows up in hover and in `cp explain`.
