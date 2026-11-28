# Values and numbers

| | |
|---|---|
| Kind | Language design |
| Owner | spec |
| Leads | `spec/core/02-values-types.md`, `spec/core/03-numbers-layout.md` |
| Describes | `spec/core/02-values-types.md`, `spec/core/03-numbers-layout.md`, `spec/conformance/core-02/`, `spec/conformance/core-03/` |

What a value is in CP Lang, how it copies or moves, and how numbers are
computed and stored. Terms follow `spec/GLOSSARY.md`; a *domain* is a
finite index space such as `0..n`.

## The problem

Contest code loses time to numbers in two opposite ways. Narrow types
overflow silently:

```cpp
int a = 100000, b = 100000;
long long s = a * b;          // overflows in int before it widens
vector<int> par(n, -1);       // -1 as "no parent": a sentinel the type can't see
```

The usual fix, `#define int long long`, doubles every array and makes
cache-bound solutions slow. Python never overflows but has no layout
control at all, and every value is a reference, so `b = a` shares a list.
Conversions add a third kind of bug: C++ converts `size_t`, `int`,
`double` and `char` into each other without a word.

## The model

```cp
let k = 3
let a = [1, 2]
let b = a.clone()  // a heap value copies only when you say so
print(k + b.len)

let x: ?Int = None  // an optional; None costs no extra bytes here
print(x ?? 7)

type Z = Mod<7>
print(Z(5) + Z(3))
```

What a contestant holds in their head:

1. **Every value computes as a 64-bit `Int`**; storage takes the fewest
   bytes the type allows (chapter 3, model). Locals are always wide.
2. **Widths come from ranges and domains, never from width names.**
   `[V]1..=1e9` stores 4 bytes per element, a position of `domain V = 0..n`
   is 4 bytes when `n` fits; there is no `U8` or `i32` to choose.
3. **`?T` usually costs nothing**: `None` lives in a value `T` can never
   hold (a spare value), such as `INT64_MIN` for `Int`.
4. **Overflow is never silent in a checked build**: arithmetic that
   leaves `Int` and a store that leaves its range both trap.
5. **Nothing is implicit**: no numeric conversion, no deep copy, no
   truthiness. Plain values copy; heap values move; `.clone()` copies.

## Key decisions

### Four kinds of value, and only `.clone()` copies heap data

**Chose:** values are *plain* (no heap part: numbers, positions,
optionals and records of plain values), *heap* (`List`, `Map`, `Str`,
arrays, structs holding them), *fns* and *generators* (R02.1.1 to
R02.1.4). Plain values copy, heap values move, `.clone()` is the explicit
deep copy (decision 128).

**Lost:** C++'s implicit deep copy on `=` (hidden \(O(n)\)); Python's
shared references (`b.append` changes `a`). Why: costs are visible, and
the one surprising case, using a moved value, is a compile error that
names `.clone()`. The full model is in
[ownership.md](ownership.md).

<!-- rejects: E0608 -->
```cp
struct Bag: xs: List<Int>
let a = [1, 2]
let b = Bag(a)          // a moves into the Bag
print(a.len, b.xs.len)  // error E0608: a was moved
```

### Constructors: generated ones store, custom ones lend (decision 147)

**Chose:** a generated constructor (`Edge(a, b, 5)`) stores its
arguments, so an owned heap argument moves in. A custom constructor is an
ordinary static fn (`SegTree(max, a)` is `SegTree.new(max, a)`): its
parameters are lent for the call, and its result owns its data unless the
signature says `lends`. A structure that keeps a copy pays for it once,
in its documented \(O(n)\) build (decision 147; R02.6.3).

```cp
var a = [5, 1, 4]
let st = SegTree(max, a)  // a is lent for the build, then free again
a[0] = 9
print(st.fold(..), a[0])  // 5 9: st owns its own copy
```

**Lost:** derived structures that keep a lend of their input, so
`LcaTable(t)` would pin `t` for as long as the table lives. Such a result
could not be returned, stored beside `t` or outlive it, and every caller
would have to reason about a borrow the contestant never asked for. Why:
`lends` in a signature then always reads "\(O(1)\), borrows", a view or an
adapter, and everything else is an ordinary owned value.

### `Int` is the one integer, 64-bit, with the minimum reserved

**Chose:** one signed 64-bit `Int` for all arithmetic; `INT64_MIN` is not
an `Int` value and is the spare value for `?Int` (decision 49; R03.2.2).
`I128` exists for the rare product that needs it.

**Lost:** C++'s menu of `int`, `long long`, `unsigned`, `size_t`
(decision 149 calls choosing a width per variable "the wrong question");
arbitrary precision as in Python (slow, and no layout). Why: 64 bits
holds every contest intermediate that isn't a deliberate 128-bit product,
and one integer type means no conversions between them.

### `/` and `%` are Euclidean (decision 347)

**Chose:** `a == (a / b) * b + a % b` with `0 <= a % b < abs(b)`, for
every sign of `a` and `b` (R01.5.2). For a positive divisor that is floor
division, so `-7 / 2` is `-4` and `-7 % 2` is `1`; for a negative one,
`7 / -2` is `-3` and `7 % -2` is `1`.

```cp
print(-7 / 2, -7 % 2, 7 / -2, 7 % -2)  // -4 1 -3 1
```

**Lost:** C++'s truncation (`-7 % 2 == -1`, so `a[i % n]` with a negative
`i` reads out of bounds); floor division with a remainder that takes the
divisor's sign (Python's rule: `7 % -2 == -1`, so "`%` is never
negative" would fail for a negative divisor). Why: a never-negative `%`
is what indexing, cyclic shifts and modular code rely on, and Euclidean
division keeps it for every divisor while changing nothing for the
common positive one. When the checker proves `a >= 0` and `b > 0`, both
lower to the plain C++ operators at no cost.

### Narrow storage from ranges, never from width names

**Chose:** a value type may be a range, such as `1..=1e9`, and storage of
that type takes the fewest bytes that hold the range. Domains do the same
for positions. Arithmetic on a ranged value gives an `Int`. Decision 149;
R03.1.1 to R03.1.9.

```cp
let n = 3
domain V = 0..n
let a: [V]1..=1000000000 = [V] v => 7  // 4 bytes per element
var s = 0
for x in a: s += x  // each x computes as Int
print(s)
```

`U8` and friends are not types; the error says to write the range
(R03.1.5):

<!-- rejects: E0901 -->
```cp
let a: [4]U8 = [1, 2, 3, 4]  // error E0901: write 0..=255
```

**Lost:** explicit width types (C++, Rust): a width says nothing about
which values are legal, so it is both less safe and more to choose.
Implicit array widening, where an array of narrow values silently
becomes an array of `Int` when an op needs it (decision 246): it hid a
doubling of memory. Why: the range is a fact the contestant already
knows from the statement ("\(1 \le a_i \le 10^9\)"); writing it once gives
both the layout and a checked bound.

### Statement facts are types: literal unions, `Str of`, field defaults (decisions 134, 135, 381)

Statements say "each query is `add x` or `del x`" and "the string has
only `0`, `1` and `?`". Those facts get the same treatment as an integer
range: written once in a type, validated on read, used by `match`.

```cp
// input: "add 5\ndel 2\n0?10\n"
var total = 0
for _ in 0..2:
    let (op, x) = read<("add" | "del", 1..=9)>()
    match op:
        "add" => total += x
        "del" => total -= x
let s = read<Str(len: 1..=10) of "01?">()
print(total, s.len)
```

**Chose:** a literal-union type such as `"add" | "del"` or `'+' | '-'`
has exactly those values; a read rejects any other token and `match` is
exhaustive over it (decision 135; R02.2.2). `Str of "01?"` is the string
form of the same idea: its characters come from the literal's alphabet,
and `Str(len: 1..=2e5)` bounds the length like a range (decision 134;
R02.3.2). The alphabet is an ordinary string literal with its escapes, so
`Str of "./\\"` allows `.`, `/` and `\` (decision 381). Struct fields may
have defaults, and a constructor may omit them (decision 134; R02.6.2):

```cp
struct Acc:
    total = 0
    steps: List<Int> = []
let b = Acc(total: 5)
print(b.total, b.steps.len)
```

**Lost:** an enum for every token set (two declarations for a fact the
statement says in one line, decision 498 keeps the enum for a token
followed by a payload); validating strings with a hand-written loop after
the read; a raw-literal alphabet, which could not name `"` or a newline
and read differently from every other string. Field defaults remove
constructor noise for accumulator records; `{k: v}` map literals,
`[x; n]` repeat lists and `(let a = ...; e)` block expressions were
rejected from the same batch of forms because an existing spelling
already says them (`[k: v]`, `[x] * n`, a block lambda).

### Structures store what their op produces

**Chose:** a `SegTree(max, a)` over 4-byte elements stores 4-byte nodes,
because `max` never leaves its inputs' range; `SegTree(+, a)` stores
`Int` nodes because sums can (chapter 3 §1, "Ops decide what a structure
stores"). `cpl explain` shows every width chosen.

**Why:** the layout a careful C++ contestant would pick by hand, with no
annotation, and no way to get a sum that overflows a narrow node.

### Spare values make optionals free

**Chose:** `?T` stores `None` in a value `T` can't hold: all ones for a
position, `INT64_MIN` for `Int`, one past the end for a range, the spare
value of a field for a record (R03.2.1 to R03.2.8). Only a `T` with no
spare value pays a tag. Decision 149.

```cp
domain V = 1..=4
struct Edge: to: V, w: Int
var best: [V]?Edge = [V].filled(None)  // 16 bytes per slot, not 24
var span: ?(lo: Int, hi: Int) = None   // None lives in lo
print(yes(best[1] is None), yes(span is None))
```

**Lost:** C++ sentinels like `-1` (the type can't see them, so `par[v]`
used as an index compiles); `std::optional` (a tag byte plus padding,
often doubling an array); a user-declared niche trait (rejected: the
compiler can find spare values itself). Why: `?V` is the right type for
"parent or none", and if it cost memory contestants would go back to `-1`.

An optional is not a sequence of zero or one element, either
(decision 464). Rust and Scala let you loop over one; here `for v in x` is an error
naming `if let`. Everything that walks also prints and folds, and an
optional deliberately doesn't print (R10.3.7): if it walked, `print(x)` on
a forgotten `None` would print nothing, the same silent bug as a stray
`-1`. When you really want "the parent, if any, then the children", say
so: `chain([t.parent(v)].present(), t.children(v))`.

### Checked builds trap; release builds don't pay

**Chose:** in a checked build (`cpl run`, stress runs, local `-DLOCAL`
builds of the emitted C++) every `Int` overflow and every out-of-range
store traps with the line. The release build sent to the judge omits
these checks by default (decision 148; chapter 3 §4). A store of a value
already proved in range is not checked at all (decision 100).

```cp
var a: [3]0..=9 = [3].filled(0)
var x = 4
a[0] = x * 2  // 8 fits; 10 would trap at this line
print(a[0])
```

**Lost:** wrapping arithmetic (C++ `unsigned`, Rust release): a silent
wrong answer; checks always on: a constant-factor cost on the judge for
bugs the checked runs already caught. Why: "zero-cost safety": the
checks exist where they find bugs and cost nothing where they'd only
cost time.

### Modular numbers are their own type

**Chose:** `Mod<P>` (or `Mod(p)` with a modulus read at run time) is a
number type whose operators reduce; mixing it with a different modulus
is a type error, and an `Int` operand is converted (chapter 3 §5).

```cp
let p = 7
type Z = Mod(p)
let a: Z = 5
print(a * 3, a + 1)
```

**Lost:** `% MOD` after every operation (a forgotten reduction is a
classic wrong answer). Why: "types carry the proofs".

### Inference is local; conversions are spelled

**Chose:** a type is inferred within one expression from its context,
never from later uses (decision 131; R02.10.1 to R02.10.4). No
conversion is implicit: `Int(x)`, `F64(n)`, `floor(x)`; there is no
truthiness, and there is no method overloading (decision 127; R02.11.1
to R02.11.3). `Int(...)` of a subset or pair value gives its index
(decision 291).

```cp
let x = F64(7) / F64(2)
let y: Int = floor(x)
print(y, ceil(x), round(x), trunc(-x))
```

**Lost:** whole-function inference (Hindley-Milner, Rust's): an error far
from the cause, and types that change when a line is added below. Why:
every type can be read off the line it's on.

### Patterns are read by case (decisions 157, 230, 369, 111)

**Chose:** in a pattern, a capitalized name is a variant or a constant
and a lowercase name always binds. So every enum variant is `PascalCase`,
`Some` and `None` included (decision 157; R01.3.1). A `const` may still
be lowercase, like std's `inf` and the operation values `add` and
`affine`; a pattern can't test against it, so a guard compares it
(decision 369; R01.3.2). A record pattern may omit a field's sub-pattern
to bind the field's own name, the mirror of the record shorthand
`(total:, take:)` that builds one (decision 230; R02.7.6). A header binds
with `if let` and `while let`; `x is pat` is the `Bool` expression for
value position, and `if x is p:` in a header gets lint L0207 with a fix
to the `let` form (decision 111; R01.7.1).

```cp
enum Dir = L | R
const inf = 1000000000
let (total:, take:) = (total: 7, take: 2)
match R: L => print(-1), R => print(total * take)
let x = 5
match x:
    v if v == inf => print("unreachable")
    v => print(v)
let m: ?Int = Some(4)
if let k = m, k > 3: print(k)
print(yes(m is Some(_) and x > 1))
```

**Lost:** lowercase variants (`some`, `none`), where a mistyped or
out-of-scope variant silently becomes a fresh binding that matches
everything (Rust lints for it; Haskell and OCaml make case part of the
grammar, as here). Uppercase-only constants (`INF`, `ADD`), rejected
because operation values read as values (`act: add`). Both `is` and `let`
in headers: once `is` could bind, programs mixed the two spellings, and
one spelling per position keeps headers uniform without losing the
expression form.

### Smaller calls

- **Custom destructuring** lost (decision 132): views are methods
  returning enums, so patterns stay core-only.
- **Members that compute without `()`** lost (decision 140): parentheses
  mean work, so `g.transpose()` looks like the \(O(n)\) it is.
- **An enum for every token set** lost (decision 498): tokens that carry
  nothing read as a literal union, `"add" | "del"`; the enum is for a
  token followed by its payload.
- **Path lengths in the weight's range** lost (decision 315): a sum of
  weights is an `Int`, like any arithmetic on a ranged value.

## Worked examples

**Widths chosen from the input** (chapter 3 §6, `core-03/spec-block-widths`):

```cp
let n = 3
domain V = 0..n
let a: [V]1..=1000000000 = [V] v => Int(v) + 1
var par: [V]?V = [V].filled(None)  // 4 bytes, None = all ones
var mx = SegTree(max, a)           // 4-byte nodes
var sm = SegTree(+, a)             // 8-byte nodes
print(mx.fold(V.range), sm.fold(V.range))
```

**A local never narrows**: the sum of narrow elements is an `Int`.

```cp
domain V = 0..4
let a: [V]1..=1000000000 = [V] v => 1000000000
print(a.sum())
```

# Reference

## Where the precise rules are

| Topic | Spec | Rule ids |
|---|---|---|
| The kinds of value | `spec/core/02-values-types.md` §2.1 | R02.1.1 to R02.1.4 |
| Scalars, strings | §2.2, §2.3 | R02.2.1 to R02.3.3 |
| Optionals | §2.4 | R02.4.1 to R02.4.8 |
| Tuples, records, structs, enums | §2.5 to §2.7 | R02.5.1 to R02.7.6 |
| Arrays and lists | §2.8 | R02.8.1 to R02.8.5 |
| Fresh types | §2.9 | R02.9.1 to R02.9.6 |
| Inference, never implicit | §2.10, §2.11 | R02.10.1 to R02.11.3 |
| Comparison with C++, Python, Rust | §2.12 | |
| Where widths come from | `spec/core/03-numbers-layout.md` §1 | R03.1.1 to R03.1.9 |
| Spare values, layout | §2 | R03.2.1 to R03.2.8 |
| Operations on niche types | §3 | R03.3.1 to R03.3.4 |
| Overflow per build | §4 | R03.4.1, R03.4.2 |
| Modular numbers | §5 | R03.5.1 |

Decisions: 1, 49, 100, 111, 127, 128, 131, 132, 134, 135, 140, 147, 148,
149, 157, 159, 230, 246, 291, 347, 357, 369, 381 in `decisions.md`.

## Changes

- 2026-11-23: first version.
