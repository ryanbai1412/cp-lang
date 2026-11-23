# 2. Values and types

Status: draft for 0.3.0. Numbers, ranges, widths, niches and layout are
chapter 3; positions, cuts and domains chapter 4; ownership chapter 5;
traits, generics and row types chapter 8. Sources: decisions 1.x, 2.x,
33, 88, 124, 128, 131, 132, 134-136, 141, 149; archived
`design/spec/11-13, 23`.

## The model in one paragraph

Every value is plain, heap or a function. Plain values (numbers, `Bool`,
`Char`, positions and cuts, `?T` of a plain `T`, and tuples, records,
structs and enums built from them) copy and fit in registers. Heap values
(`List`, `Str`, arrays, maps, every container) move, and copy only with
`.clone()` (decision 128). Function values capture nothing at zero size
and are always inlined. `?T` is the one way to say absent, stored in a
spare value of `T` (chapter 3). Inference unifies within one expression,
never across statements (decision 131), so a type error points at the
line that caused it.

## 2.1 The kinds of value

Every value is one of:

- a **plain value**: numbers, `Bool`, `Char`, positions and cuts of a
  domain, `?T` of a plain `T`, and tuples, records, structs and enums
  built only from plain values. Plain values copy on assignment and fit
  in registers or a few words. (R02.1.1)
- a **heap value**: `List<T>`, `Str`, `[D]T` arrays, `Map`, `Set`, every
  library container, and any struct holding one. Heap values move on
  assignment and are copied only by `.clone()` (decision 128). (R02.1.2)
- a **function value**: `fn` items, lambdas and operation values. They
  are zero-size when they capture nothing and are always inlined; there
  is no boxed function type (chapter 13). (R02.1.3)
- a **generator value**: a recipe that pushes elements to a loop body
  (chapter 6, `07-sequences.md`).

Whether a value is owned or lent is not part of its type (chapter 5). (R02.1.4)

## 2.2 Scalars

| Type | Values | Notes |
|---|---|---|
| `Int` | `-(2^63 - 1) ..= 2^63 - 1` | the default integer; `Int.MIN` is one above `-2^63`, which is `?Int`'s `None` (decision 49) |
| range types `lo..=hi`, `lo..hi`, `lo..`, `..=hi` | the integers in the range | an `Int` with a contract and a storage width (chapter 3) |
| `U64` | `0 ..= 2^64 - 1`, wrapping | hashes, masks, bit tricks |
| `I128` | signed 128-bit | products past `Int` |
| `F64` | IEEE double | `==` is exact; `close(a, b, eps:)` is the tolerant compare |
| `Bool` | `true`, `false` | 1 byte; packed in `Bits`; `Int(b)` is 1 or 0 |
| `Char` | ASCII `0..=127` | 1 byte; `Int(c)`, `Char(i)`, `c.is_digit`, ... |
| `()` | the unit value | what statements and `print` return |
| `Never` | no values | the type of `return`, `break`, traps, `todo` |

There are exactly four number types (decision 1.7). Narrow storage is a
range type, never a separate integer type: `[V]0..=255` stores one byte
per element. (R02.2.1)

Literal-union types are scalars too: `'+' | '-'` or `"add" | "del"` has
exactly those values, reads validate them, and `match` is exhaustive over
them (decision 135). (R02.2.2)

## 2.3 Strings

`Str` is a flat sequence of `Char` indexed by `0..len`, with the sequence
capabilities of chapter 7 (it is `Indexed`, `Sized` and `Bidir`). Slices
keep source positions, as for every sequence (`04b-keys.md`). (R02.3.1)

- `Str of "01?"` is a string whose characters come from the literal's
  alphabet. The alphabet is a domain, so `[Alpha]Int` is a table over it
  and reads validate it. `Str(len: 1..=2e5)` bounds the length, checked
  on read like a range (decision 134); the bounds may be runtime values, so
  `read<Str(len: n..=n)>()` checks an exact length. (R02.3.2)
- `+` concatenates (allocating; the cost is visible as a new value);
  `s.push(c)` appends in place. (R02.3.3)
- Interpolation `"{x}"` formats through the `Show` trait (chapter 10).

## 2.4 Optionals

`?T` is `T` or `None`.

- `x.is_some_and(p)` tests a value and a predicate at once (`false` on `None`);
  `x.is_none_or(p)` is its dual (`true` on `None`). `x.map(f)` and `x.and_then(f)`
  (for `f: T -> ?U`) transform the value and keep `None`. (R02.4.1)
- `T` converts to `?T` implicitly; the other way is explicit: `x ?? d`
  (lazy fallback), `x!` (unwrap or trap, naming the place), `if let`,
  `while let`, `let ... else`, `match`. (R02.4.2)
- When `T` has a spare value (a niche), `?T` takes no more bytes than `T`
  (chapter 3). Otherwise it adds a flag byte, rounded to alignment.
  `?(?T)` is legal and adds a flag around `?T`. (R02.4.3)
- **Arithmetic and bitwise operators lift**: `?Int + Int` is `?Int`,
  `None` if either side is. Lifting is three ordinary impls, not a
  language rule (`impl<T: Add> Add for ?T`). (R02.4.4)
- **Ordering does not lift**: `?T` is not `Ord`, so `dp[j] + c < best`
  with an optional left side is an error, never a silent `false`. `==` is
  structural (`None == None`). (R02.4.5)
- `min`, `max`, `min=` and `max=` treat `None` as the identity: a `None`
  target takes the right side, a `None` right side changes nothing. With
  lifted arithmetic this is "skip unreachable states": (R02.4.6)

  ```
  dist[v] min= dist[u] + w          // dist[u] none: nothing happens
  ```

- **Postfix `x?`** propagates where operators don't reach: `a[x?]`,
  `f(x?)`, `x?.len`, `(dp[j]? + w, j)`. The extent is the nearest
  enclosing parenthesized group, call argument, left operand of `??`,
  lambda or `fn ... =` body, or statement right-hand side. A `?` in an
  `if` or `while` condition is an error (write `... ?? false`), and a `?`
  whose result is dropped is an error. `?` is never control flow; an
  early exit is `let x = e else: return None` (chapter 6). (R02.4.7)
- `x!` is the `Unwrap` trait. `?T`, `[D]?T` and `List<?T>` implement it;
  the array forms check every element in one pass and trap naming the
  first gap ("depth[17] is None"). This ends a fill phase:
  `let depth = depth!`. `.all_some()` is the non-trapping form. (R02.4.8)

## 2.5 Tuples and records

- Tuples `(Int, V)` are positional; `.0`, `.1` and destructuring read
  them. (R02.5.1)
- Records `(lo: Int, hi: V)` are anonymous structs with named fields.
  Two record types are equal when they have the same field names and
  types in the same order. Records are structural: a record with fields
  `.to` and `.w` satisfies field traits such as `Target<V>` and
  `Weighted<W>` without an impl (chapter 8). (R02.5.2)
- Record update copies with replacements: `a with to: u` or
  `(..a, to: u)`. (R02.5.3)
- Row types `R: (u: V, v: V, ..)` accept any record with at least those
  fields, and `..R without (u, v)` names the rest. There is no record
  subtyping and no implicit field dropping: a function that wants fewer
  fields says so with a row bound (chapter 8). (R02.5.4)
- A record pattern names fields: `(u: a, v: b)` binds `a` and `b`, and
  `(u:, v:)` is shorthand for `(u: u, v: v)`, in `let`, `for`, `match`
  and parameters. Fields not named must be written as `..`:
  `(u:, ..)`. (R02.5.5)

## 2.6 Structs

```
struct Edge: u: V, v: V, w: Int

struct Tally:
    var top = 0                     // typed by its default
    var sum = 0
    name: Str = ""
    fn add(mut self, x: Int):
        self.top max= x
        self.sum += x
```

- Fields are fixed after construction unless declared `var`; assigning a
  non-`var` field is an error naming the field (decision 136). (R02.6.1)
- Constructors are generated: `Edge(a, b, 5)` or `Edge(u: a, v: b, w: 5)`;
  fields with defaults may be omitted (decision 134). A custom
  constructor is a static fn (`Graph.read()`). A `type` alias is the type
  it names everywhere: `type Z = Mod<1e9 + 7>`, then `Z(a)`; `type G =
  Subset<R>`, then `for s in G` walks the subsets. There are no
  constructor side effects and no destructors. (R02.6.2)
- **Two kinds of constructor, one rule each.** A generated constructor
  (`Edge(a, b, 5)`, a struct literal) stores its arguments, so it takes
  them by value: an owned heap argument moves in, a lent one makes the
  struct lent. A custom constructor is an ordinary static fn
  (`SegTree(max, a)` is `SegTree.new(max, a)`, `Rooted(g, root: r)`): its
  parameters are lent for the call and its result is owned unless the
  signature says `lends` (chapter 5, decision 147). The cost rule tells
  them apart: a custom constructor that keeps data copies it (\(O(n)\),
  documented) or says `lends`. When a type declares a static `new`,
  `T(args)` always calls it, so `Mod<7>(10)` reduces to `3` and the
  field form `Mod<7>(v: 3)` is E0310. (R02.6.3)
- `struct List<T>` with no body is opaque: its fields are private to its
  module. A struct body may hold methods with `self` or `mut self`. (R02.6.4)
- Named structs satisfy traits only through `impl`, including field
  traits (`impl Target<V> for Edge` with no body when the field exists). (R02.6.5)
- `@get` on a read-only method whose other parameters all have defaults
  lets it be called without parentheses (`v.next` is `v.next(1)`); it
  must be \(O(1)\) (decision 140). (R02.6.6)
- Storage may reorder fields to remove padding; source order is what
  constructors, patterns and printing use (chapter 3). (R02.6.7)

## 2.7 Enums

```
enum Dir = L | R | U | D            // payload-free: an index type
enum Ev:
    Add(v: V)
    Ask(l: V, r: V)
    fn is_query(self) -> Bool = self is Ask
```

- Variants are `PascalCase` names in their enum's scope, like `Some` and
  `None` of `?T` (decision 157). Context resolves bare variant names in
  patterns and where the expected type is the enum; a lowercase name in
  a pattern always binds. (R02.7.1)
- `@tag(lit)` on a variant names the token it reads and prints as (`@tag("Q") Ask(h: Int)`,
  `@tag(1) Add(l: V, r: V)`); without it the token is the variant's name. Tags are distinct
  within the enum (E0407). A token set whose tokens carry nothing is a
  literal union (`"DR" | "DL"`, R02.2.2); when a payload follows the
  token, it is an enum with `@tag` (decision 357). (R02.7.2)
- A payload-free enum is a domain: `[Dir]Int` is a table, `for d in Dir`
  walks it. (R02.7.3)
- `match` is exhaustive. Patterns match only the stored shape; there are
  no user-defined patterns (decision 132). (R02.7.4)
- A record pattern names the fields it binds, `(to: u, w: c)`, and `..`
  skips the rest; `(total:, take:)` binds each field to its own name, as the
  record expression `(total:, take:)` builds one. It works wherever a pattern
  does: `let`, `for`, `match` (decision 230). (R02.7.6)
- Layout uses niche filling, so `Ev` is 8 bytes with no tag when `V`
  positions are 4 bytes (chapter 3). (R02.7.5)

## 2.8 Arrays and lists

- `[D]T` is a fixed-size array indexed by domain `D` (positions of `D`,
  never raw `Int`; an array over an anonymous range such as `[n]T` or
  `[0..=n]T` takes any `Int`, checked, R04.3.3). `[R, C]T` is a dense row-major 2-D array indexed
  `g[r, c]`; `[(R, C)]T` is the same type. `g[r]` is the row place. A
  counted dimension `[n]T` means `[0..n]T`. (R02.8.1)
- `List<T>` grows, is indexed by `0..len` positions, and its slices keep
  source positions. (R02.8.2)
- `[D]T` arrays are built by tables (`[V] v => 0`), typed reads
  (`read<[V]Int>()`), comprehensions with an expected array type, or
  `[D].filled(x)`, or a scalar `x: T` where a `[D]T` is expected (a fill,
  R04.4.27); every element is initialized. (R02.8.3)
- `xs[i]` on a heap element lends it (`let row = rows[i]`); on a plain
  element it copies. (R02.8.4)
- Large arrays never live on the C++ stack (chapter 13). (R02.8.5)

## 2.9 Fresh types: domains and runtime-parameterized types

A binding can introduce a type whose identity is the binding:

<!-- spec-only: fragment: `t` and `c` come from the surrounding text -->
```
domain V = 1..=read<1..=2e5>()
let (domain Piece, piece_of) = t.components(without: c)
type Z = Mod(read<Prime>())         // runtime modulus
```

- Two fresh types are never equal, even from the same expression. (R02.9.1)
- A fresh type is block-scoped. A value whose type mentions it cannot
  leave the block by `return`, by assignment to an outer `var`, by a
  stored closure, by a per-program binding (so a position cannot leak
  into the next test case), or by a field of a struct declared outside
  the block. Passing it to a call is always fine. (R02.9.2)
- A function may return a fresh type existentially:
  `fn components(...) -> (domain P, [V]P)`; the caller binds it with
  `let (domain P, ...) = ...`. User code can write such functions: the
  body returns a tuple whose `domain` slot is a domain declared in the
  body, `(J, [J] j => 0)`. A struct return may leave a domain parameter
  as `_`, `-> Rooted<V, _>`: the body's value fixes it, and the caller
  reads it as a member, `t.Pre`. The signature leaves the body's base out,
  so a function returning a fresh domain documents it; every one in
  `std` is counted from 0 (`0..k`), so `Int(c)` runs `0..C.count`
  and 1-based output adds 1. (R02.9.3)
- A fresh type carries runtime data (a domain's count, a modulus) in one
  static slot per declaration site, saved and restored when a recursive
  block re-enters. Generic code reads the slot; nothing is passed
  hidden. A compile-time constant specializes the slot away. (R02.9.4)
- `V.count` is typed by the declared range, which is where storage
  widths and overflow proofs come from. (R02.9.5)
- Product domains `domain State = (V, K)` and disjoint unions
  `domain Node = In + Out` are domains too (decisions 2.x, 88). Written
  inline, `(V, K)` is the anonymous product domain, the same one that
  `[V, K]T` is indexed by: `(V, V).all((u, v) => d[u, v] == d[v, u])`
  (decision 159). (R02.9.6)

## 2.10 Type inference

- Inference is local. Siblings in one expression (list elements,
  `if`/`else` branches, `match` arms, comprehension results) are unified
  before generics are fixed, with `None` joining `T` to `?T`. Inference
  never looks at later statements: `var x = None` followed by `x = 3` is
  an error asking for `?Int` at the declaration (decision 131). (R02.10.1)
- `let` keeps the computed range of its initializer; an unannotated
  `var` widens to `Int`; `var x: lo..=hi` checks each assignment.
  Range inference goes through arithmetic on `let`s, runs no loop
  fixpoints and crosses no calls. A local's range type may have runtime
bounds (`var i: 1..=n = read()`). (R02.10.2)
- Function signatures are written in full except that a single-capital
  type parameter used in a parameter type may be left undeclared
  (`fn peel(s: [D]Bool)`). (R02.10.3)

## 2.11 Never implicit

- No implicit numeric conversions except into a wider range of the same
  type, `T` into `?T` and `Int` into `Mod<P>` (R04.4.22-28). `Int` to `F64`
  is `F64(x)`; `F64` to `Int` names its rounding: `floor(x)`, `ceil(x)`,
  `round(x)` or `trunc(x)`. `Int(x)` of an `F64` is an error (E0302)
  whose fix offers `floor(x)`. (R02.11.1)
- No implicit copies of heap values (`.clone()`), no implicit
  allocation behind an \(O(1)\)-looking member, no implicit `Int` to
  position (`V(x)` traps out of range; `V.at(x)` is `?V`). (R02.11.2)
- No inheritance, no subtyping, no overloading by argument type
  (decision 127), no user-defined implicit conversions. (R02.11.3)
