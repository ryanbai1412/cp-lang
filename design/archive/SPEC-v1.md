# The CP Language and Standard Library Specification (v1)

## Contents

1. Design Philosophy and Core Principles
2. Lexical Structure
3. Bindings and Functions
4. Types and Representation
5. Index Domains
6. Arrays, Tables, Lists, Slices, and Views
7. Ranges
8. Control Flow and Patterns
9. Mutation, Ownership, and Borrowing
10. Abstraction: Traits, Operation Values, and Generics
11. Program Structure
12. Typed Input and Output
13. Testing Hooks and Stress Testing
14. Safety Model: Static Proofs, Debug Traps, and Release
15. Lowering to C++17
16. Standard Library Specification (`std.seq`, `std.col`, `std.alg`, `std.graph`, `std.tree`, `std.str`, `std.math`, `std.geo`, `std.io`/`std.interactive`, `std.rand`)
17. Resolution of Open Questions from `00-grounding.md`

---

## 1. Design Philosophy and Core Principles

CP is a statically typed, compiled programming language designed for
competitive programming, targeting single-file C++17 output for online judges.
Its central design rule is simple: **the obvious way to write a solution should
be the correct and fast way.** Every construct in the language is tested
against a single question: *could a programmer write this line wrong in a way
that still compiles?* Whenever the answer is yes, the language either eliminates
the mistake at compile time or replaces the error-prone idiom with a named,
zero-cost abstraction.

Six principles govern every decision in this specification:

- **Low mental overhead, high compositional fit.** Solutions should read like
  the one- or two-sentence editorial idea, not like pointer arithmetic or
  boilerplate container plumbing. Pieces compose uniformly: domains index
  arrays, segment trees, graphs, and DP tables alike; ranges feed prefix sums,
  slices, binary search, and two pointers alike.
- **Make whole bug classes unrepresentable (P0).** Off-by-one shifts, indexing
  the wrong array, stepping off a grid or array boundary, confusing inclusive
  and exclusive endpoints, silent 32-bit overflow, negative `%` remainders,
  sentinel collisions (`-1` vs valid values), and forgotten per-test resets are
  prevented by typed index domains, boundary domains (`I.bounds`), half-open
  vs closed ranges (`..<` vs `..=`), range/refinement types, niche optionals
  (`?T`), and file-body `@multitest`.
- **Zero-cost lowering under 64 KB.** Every language and standard library
  abstraction lowers to C++17 code as fast as careful hand-written C++, with
  the complete emitted submission guaranteed to stay under the standard 64 KB
  judge source limit via tree-shaking and layout-sharing monomorphization.
- **Zero hidden magic.** Representation always follows the declared type: a
  `SortedSet<V>` never silently becomes a bitset, a `[V]Bit` never silently
  bit-packs, and an O(n log n) algorithm is never silently swapped for an O(n)
  one. Every algorithm is named explicitly (`bfs`, `bisect`, `sliding`,
  `falling`, `SegTree`), and every heap allocation or O(n) copy is visible in
  the source.
- **Multitest and single-test programs feel identical.** A program file is a
  function body. Adding `@multitest` at the top runs the file body fresh per
  test case; `static let` lifts one-time precomputation outside the test loop.
  No reset boilerplate is ever written.
- **Small principled core; contest shapes in the standard library.** The core
  language contains only general mechanisms: domains, range types, optionals,
  generators, traits, operation values, ownership, and decorators. Contest
  shapes — grids, rings, implicit heaps, segment trees, graphs, trees, prefix
  sums — live in the standard library as ordinary reusable modules.

### Standard Library Inclusion Criterion

A type or function belongs in the standard library if and only if all three
conditions hold:

1. **Reusable textbook shape or algorithm.** It names a general mathematical
   or algorithmic structure cited across many unrelated problems (such as
   `grid`, `ring`, `prefix_sums`, `bisect`, `sliding`, `SegTree`, `Tree`,
   `FlowGraph`), never a single-problem pattern (such as "sum of distances to a
   median" or "max DP plus range max").
2. **Eliminates a concrete bug class.** It encapsulates error-prone index
   translation, boundary handling, recursion depth, sentinel logic, or
   anti-hack hashing.
3. **Single transparent cost model.** It has one documented time and memory
   complexity with no hidden algorithmic trade-offs.

---

## 2. Lexical Structure

### Encoding, Indentation, and Blocks

Source files are UTF-8 (file extension `.cp`). Blocks are indentation-based,
introduced by a trailing colon `:`. Indentation uses ASCII spaces only; tab
characters anywhere in leading indentation are a compile error. Every dedent
must align with an enclosing block's indentation level.

Single-statement blocks may appear on the same line after `:` when short:

```cp
if n == 0: return
for v in V: deg[v] += 1
```

Newlines terminate statements, except inside parentheses `(...)`, brackets
`[...]`, or braces `{...}`, where line breaks and indentation are ignored.
Semicolons `;` separate multiple statements on a single line (used sparingly,
typically in short update bodies or inline expression blocks):

```cp
big.set(i, v > 1); x[i] = v; prod[i] = M(v)
```

Curly braces `{ ... }` exist solely for inline multi-statement expression
blocks (most commonly inside lambdas):

```cp
edges.sort(by: e => { let d = abs(e.u - e.v); (d, e.w) })
```

### Comments

Line comments begin with `//` and run to the end of the line. There are no
nested block comments; multi-line commentary uses consecutive `//` lines.
A compiler version stamp (`// cp 1.0`) may appear on line 1 and is ignored by
the lexer.

### Identifiers and the Casing Rule

Identifiers start with an ASCII letter or `_`, followed by ASCII letters,
digits, or `_`. The language enforces a syntactic casing distinction that the
parser relies on directly:

- `snake_case` (starts with lowercase `a`–`z` or `_`): values, variables,
  parameters, fields, functions, methods, modules, and operation constants.
- `PascalCase` (starts with uppercase `A`–`Z`): types, domains, traits, enum
  variants, and type/domain generic parameters.
- `SCREAMING_SNAKE` (all uppercase with underscores): compile-time scalar
  constants such as `INF`, `MOD`, `LOG`.

**Parser rule for `<`:** In an expression, `Ident<` begins a generic argument
list if and only if `Ident` is `PascalCase` (not `SCREAMING_SNAKE`) and the
tokens up to the matching `>` parse cleanly as type/domain/const arguments
followed by a call `(`, member `.`, or scope boundary. Consequently, `INF < x`
and `a < b` always parse unambiguously as less-than comparisons without needing
turbofish syntax (`::<>`). For the rare case where a lowercase generic function
needs explicit type arguments that cannot be inferred from context, use
`f[T](args)` or annotate the receiving binding (`let x: T = f(args)`).

**Pattern safety from casing:** Inside `match` and `if let` patterns, any
`PascalCase` identifier must resolve to a known enum variant or constant; a
misspelled variant is a compile error rather than silently binding a catch-all
variable.

### Keywords

Reserved keywords are kept minimal:

```text
let  var  const  static  fn  mut  struct  enum  trait  impl  domain  pool
type  use  pub  as  in  where  if  elif  else  match  for  while  loop
repeat  break  continue  return  yield  defer  undo  do  with  by  and
or  not  true  false  none  some  self  Self  extern  export  protocol
unchecked  assert  requires  ensures  invariant  law
```

### Numeric Literals

Integer literals default to `Int` (64-bit signed) and support underscores for
readability, binary (`0b`), hexadecimal (`0x`), and exact scientific notation:

```cp
let a = 1_000_000_007
let b = 1e9 + 7             // Int: 1000000007 (exact integer)
let c = 2e5                 // Int: 200000
let mask = 0b1010_0110      // Int (or U64 / U32 when typed)
let hex = 0x3f3f_3f3f
```

Scientific notation `MeK` is an `Int` literal whenever its exact mathematical
value is an integer within `Int` range (for example `1e9`, `2e5`, `1e18`,
`1.5e6`). Non-integer scientific values (`1e-9`, `1.5e-4`) and literals with a
decimal point (`0.5`, `3.14159`, `1.0`) have type `Float`.

### Character and String Literals

Character literals use single quotes (`'a'`, `'\n'`, `'\''`, `'\\'`) and have
type `Char` (a single ASCII byte).

Every double-quoted string literal supports interpolation with `{expr}`. A
literal brace is written `{{` or `}}`. Raw strings prefixed with `r"..."`
disable both escape sequences and interpolation:

```cp
print("Case #{tc}: {ans}")
print("value = {x:.9}, padded = {k:05}, hex = {m:x}")
let raw = r"literal {braces} and \n"
```

Format specifiers follow a colon inside `{...}` using standard `printf`-style
specifiers: `.N` for fixed decimal digits on `Float` (`{x:.6}`), `0W` for
zero-padded width (`{n:05}`), `>W` / `<W` for right/left alignment, `x`/`X` for
hexadecimal, and `b` for binary. Every interpolated expression must implement
`Show`.

### Operators and Precedence

From tightest to loosest precedence:

1. Postfix: member `.x`, call `f(...)`, index `a[i]`, unwrap `!`, propagate `?`
2. Exponentiation (right-associative): `**` (`2 ** 20`, `M(2) ** n`)
3. Unary prefix: `-` (arithmetic negation), `~` (bitwise NOT)
4. Multiplicative: `*`, `/`, `%`
5. Additive: `+`, `-`
6. Bitwise shifts: `<<`, `>>`
7. Bitwise AND & range intersection: `&`
8. Bitwise XOR: `^`
9. Bitwise OR: `|`
10. Range constructors: `..<`, `..=`, `..` (and step `by`)
11. Optional coalescing: `??` (binds looser than arithmetic, so `dist[v] ?? -1`
    and `v.next.map(u => dp[u]) ?? 0` need no parentheses; when adding a
    coalesced optional to another term, write `(a ?? 0) + b`)
12. Comparisons (chained, non-associative): `==`, `!=`, `<`, `<=`, `>`, `>=`,
    `in`, `not in`. Chained comparisons `lo <= x < hi` evaluate `x` once and
    desugar to `lo <= x and x < hi`.
13. Logical negation: `not`
14. Logical conjunction: `and` (short-circuiting)
15. Logical disjunction: `or` (short-circuiting)
16. Functional update: `with`
17. Assignment & compound assignment (statements, not expressions): `=`, `+=`,
    `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`, `min=`, `max=`.

Bitwise operators `&`, `^`, `|` are intentionally looser than `+`/`-` but
tighter than comparisons (`(mask & (1 << i)) != 0` can be written
`mask & (1 << i) != 0` safely — eliminating C++'s infamous `a & b == 0`
precedence bug!).

---

## 3. Bindings and Functions

### Bindings: `let`, `var`, `const`, `static let`

- **`let`** introduces an immutable binding. Once initialized, it cannot be
  reassigned or mutably borrowed.
- **`var`** introduces a mutable binding.
- **`const`** introduces a compile-time constant evaluated during compilation
  and embedded in the generated C++. Its initializer cannot read input or call
  nondeterministic functions. Embedded tables larger than 16 KB emit a compiler
  warning to protect the 64 KB submission budget.
- **`static let`** introduces an immutable binding evaluated once at runtime
  on first entry and retained for the lifetime of the process. Under
  `@multitest`, `static let` shares precomputation (sieves, factorial tables,
  NTT roots) across all test cases. There is intentionally **no `static var`**:
  mutable cross-test state is banned because it is the #1 source of multitest
  bugs.

```cp
const MOD = 998_244_353
const SMALL_PRIMES = List(p for p in 2..=100 if is_prime(p))

static let sieve = Sieve(2e6)               // computed once across all tests

let (n, k): (1..=2e5, 1..=n) = read()       // destructuring immutable read
var best = 0                                // mutable local
let _ = pop_unused()                        // explicit discard
```

Type annotations are optional when inferable from the initializer, and required
on `read()` calls or uninitialized/empty containers whose element type is not
yet fixed. A `var` declaration for a domain array may omit an initializer only
when its element type is `Zero` or `?T` (defaulting to zero / `none`), or it
may give an explicit fill value (`var dp: [V]Int = 0`, `var dist: [V]?Int = none`).

### Functions and Methods

Functions are declared with `fn` and support both multi-line indented bodies
(where the final expression is the return value) and single-line `= expr`
bodies:

```cp
fn sq(x: Int) -> Int = x * x

fn clamp(x: Int, lo: Int, hi: Int) -> Int:
    if x < lo: lo
    elif x > hi: hi
    else: x
```

Functions with no return value omit `-> R` (returning `Unit`). The `return`
statement exists for early exits (and works at the top level of a program file
as well).

### Parameters: Positional, Named, Defaults, Named-Only, Variadics

Every parameter may be passed either positionally or by name. Default values
are evaluated at each call site. A semicolon `;` in the parameter list marks
all subsequent parameters as named-only — used heavily in the stdlib so call
sites self-document (for example `bfs(from: seeds, next: ...)` and
`Tree<V>(edges).rooted(at: V.first)`). Typed variadic parameters use `..xs: T`
and are received as a slice view:

```cp
fn connect(u: V, v: V; weight: Int = 1, directed: Bool = false):
    ...

connect(a, b, weight: 5)
connect(a, b, directed: true)

fn sum_all(..xs: Int) -> Int:
    var total = 0
    for x in xs: total += x
    total
```

A mutating parameter is marked `mut` at **both** the declaration and every call
site so mutation is never invisible when reading a caller:

```cp
fn relax(mut best: Int, cand: Int):
    best min= cand

relax(mut ans, cur)
```

Method receivers (`self` or `mut self`) do not require `mut` at the call site:
`st.update(r, delta)`.

### Local and File-Level Helpers: `fn` vs `mut fn`

Because a program file is a function body, top-level `fn` declarations and
nested `fn` declarations follow one unified scoping rule:

- A plain `fn` can read any `let` or `var` declared in its enclosing function
  (or file body), provided that variable is initialized before any call to the
  `fn`. Calling a `fn` before a captured binding is initialized is a compile
  error naming both the call and the binding.
- A plain `fn` **cannot mutate** an enclosing `var`. To mutate enclosing `var`s
  (such as a DFS updating `vis` and `tin`, or `delete(v)` updating `a` in a
  heap problem), declare it `mut fn`:

```cp
var a: [Node]?Int = read()

fn larger_child(v: Node) -> ?Node =
    v.children.filter(c => a[c] != none).max_by(c => a[c])

mut fn delete(v: Node):
    var u = v
    while let c = larger_child(u):
        a[u] = a[c]
        u = c
    a[u] = none
```

A `mut fn` (and any `fn` that captures an enclosing `var`) **cannot escape**
its enclosing scope: it cannot be returned, stored in a struct, or placed in a
collection. While it executes, its captured `var`s obey standard borrow
exclusivity. Functions decorated with `@memo` cannot capture any `var` at all,
because caching across changing mutable state would produce wrong answers.

### Lambdas and `do:` Block Expressions

Lambdas use `=>`. Parameter types are inferred from context when omitted:

```cp
let double = x => x * 2
let cmp = (a, b) => a.w < b.w
let complex = x => {
    let k = key(x)
    (k.hi, -k.lo)
}
```

A `do:` block introduces a scoped expression whose final expression is its
value. Inside `do:`, `return` is not captured (it still exits the enclosing
function), while postfix `?` on an optional propagates `none` out of the `do:`
block if the `do:` block's expected type is `?T`:

```cp
let gain: ?Int = do:
    let prev = i.prev?
    let next = i.next?
    some(a[prev] + a[next] - 2 * a[i])
```

---

## 4. Types and Representation

Every type in CP has a concrete, predictable memory layout. Representation
never changes secretly behind the programmer's back.

### `Int` and Its Niche

`Int` is the default integer type: 64-bit signed, with valid range
`[-(2^63 - 1), 2^63 - 1]`.

- `Int.MIN` is `-(2^63 - 1)` (`-9223372036854775807`), **not** `-2^63`.
- The single excluded bit pattern `-2^63` (`LLONG_MIN`) is reserved as the
  niche sentinel for `?Int`.
- Consequently, `?Int` occupies exactly **8 bytes** (zero extra tag bytes), and
  `-x` and `abs(x)` can **never** overflow for any valid `Int`.
- Producing `-2^63` is an overflow, trapped in debug builds. Code that needs
  all 64 raw bit patterns uses `I64` or `U64`.

### Floor Division `/` and Non-Negative Remainder `%`

Integer division `/` rounds toward \(-\infty\) (mathematical floor division),
and `%` always produces a non-negative remainder in `0..<abs(b)` for `b > 0`:

```cp
-7 / 2          // -4
-7 % 3          // 2
(x - 1) % n     // always in 0..<n when n > 0
```

Whenever range analysis proves both operands are non-negative (the common
case), `/` and `%` lower directly to plain C++ `/` and `%` with zero extra
instructions. Division or remainder by zero traps in debug builds. For
truncating C-style division, use `trunc_div(a, b)`; for ceiling division of
non-negative integers, use `ceil_div(a, b)`.

### Range and Refinement Types

A range type constrains an integer to declared bounds, written `lo..=hi`
(inclusive), `lo..<hi` (half-open), or `lo..` (lower-bounded):

```cp
let (h, g): (2..=20, 1..=h - 1) = read()
let a: [V]0..=1e9 = read()
let c: [V]-100..=100 = read()
let p: 1..=1e18 = read()
```

Bounds may reference earlier bindings in scope (dependent bounds like
`1..=h - 1` or `0..=n * (n - 1) / 2`). Binding refinements add relational
constraints with `where`:

```cp
let qs: [Q](l: V, r: V) = read() where l <= r
```

Range types serve four roles from a single declaration:

1. **Storage-width selection (explicit, deterministic).** In arrays (`[D]R`)
   and struct fields, a range type uses the narrowest standard integer width
   that holds its static worst-case hull:
   - Fits in `0..=255` (`uint8_t`) or `-128..=127` (`int8_t`): **1 byte**.
   - Fits in `0..=65535` (`uint16_t`) or `-32768..=32767` (`int16_t`): **2 bytes**.
   - Fits in `0..=2^32 - 1` (`uint32_t`) or `-2^31..=2^31 - 1` (`int32_t`): **4 bytes**.
   - Otherwise: **8 bytes** (`int64_t`).
   For an optional range type `?R`, one spare bit pattern is reserved in the
   chosen width if available (for example `?0..=1e9` stays **4 bytes** because
   `uint32_t` has billions of unused values above `1e9`). If the range exactly
   fills a width (like `0..=255`), `?0..=255` widens to the next width (2 bytes)
   or adds a flag byte — always predictable and visible on hover.
2. **64-bit arithmetic, checked narrowing stores.** All arithmetic on range
   types promotes immediately to 64-bit `Int`, so `a[i] + a[j]` or `a[i] * a[j]`
   never wraps at 32 bits. Narrowing happens only when storing back into a
   narrow array or field, where the compiler either proves the value fits or
   emits a debug check (reported by `cp build --checks`).
3. **Compile-time overflow proving.** The compiler propagates intervals and
   simple relational facts (`<=`, `<`, `where`, `invariant`, loop iteration
   counts) through expressions and accumulators:
   - **Proven safe** (computed interval lies within `[Int.MIN, Int.MAX]`): no
     runtime check emitted. For example, with `p: [V]Point<-4e8..=4e8>` and
     `n <= 1e5`, the compiler proves `4 * (edge_sum + cross)` fits in 64 bits.
   - **Provably overflows** (worst-case interval on declared input bounds
     exceeds `Int`, e.g., summing `2e5` products of `1e9 * 1e9`): **compile
     error** naming the expression, the derived bound, and the input bounds
     responsible, suggesting `I128`, `Mod<P>`, or a tighter input bound.
   - **Unknown** (e.g., an unbounded loop accumulator whose termination depends
     on runtime values, such as `rel *= x[s]` in Horses): checked in debug
     builds, unchecked in `--release`, and listed by `cp build --checks`.
4. **Validation and stress-test generation.** `read()` validates range and
   `where` constraints in debug builds, and `cp stress` uses them to generate
   edge-biased random inputs automatically.

### Fixed-Width Integer Types

For bitmasks, hashing, exact C++ interop, and 128-bit intermediate arithmetic,
CP provides `U8`, `I32`, `U32`, `I64`, `U64`, and `I128`:

- Unsigned types (`U8`, `U32`, `U64`) use wrapping two's-complement arithmetic
  and expose bit methods: `.popcount()`, `.clz()`, `.ctz()`, `.bit(k)`,
  `.lowbit()`, `.rotl(k)`, `.rotr(k)`.
- `I64` uses the full 64-bit signed range (including `-2^63`, so `?I64` is 16
  bytes rather than 8).
- `I128` lowers to `__int128` and is only used where explicitly written, so
  performance stays predictable.
- Conversions between integer types are always explicit: `x.as<I32>()` (checked
  in debug), `x.as<I128>()` (lossless widening), `x.wrap<U64>()` (truncating /
  two's-complement wrap).

```cp
let prod: I128 = a.as<I128>() * b.as<I128>()
let h: U64 = x.wrap<U64>() * 0x9e3779b97f4a7c15
let bits = h.popcount()
```

### `Bit`, `Bool`, `Char`, `Str`, `Float`

- **`Bit`**: represents a single bit `0` or `1`, stored as **1 byte**
  (`uint8_t`). It supports bitwise `^`, `&`, `|`, `~`, and `read(digits)` reads
  `'0'`/`'1'` characters directly into `[D]Bit`. `[D]Bit` is never secretly
  bit-packed; when you want packed 64-bit word operations, use `BitSet<D>`.
- **`Bool`**: `true` or `false`, stored as 1 byte (`uint8_t`, values `0`/`1`,
  with 254 niche values so `?Bool` is 1 byte).
- **`Char`**: a single ASCII byte (`uint8_t`). Character subtraction produces a
  range-typed integer when bounds are known (`c - 'a'` is `0..<26` when
  `c: 'a'..='z'`).
- **`Str`**: an owned, contiguous sequence of ASCII bytes with O(1) indexing
  and no UTF-8 decoding overhead. Its length gives a domain `s.indices`, so all
  sequence views work directly on strings (`runs(s)`, `windows(s.indices, k)`,
  `s[l..=r]`).
- **`Float`**: IEEE-754 64-bit floating-point (`double`). Direct `==` or `!=`
  on `Float` triggers a compiler lint recommending `abs(a - b) <= eps`. Printing
  a `Float` requires explicit precision (`print(x, digits: 9)` or `"{x:.9}"`)
  rather than defaulting to 6 significant digits.

### Optionals (`?T`)

`?T` is the sole mechanism for absence, replacing `-1`, `LLONG_MAX`, and
`nullptr` sentinels. Values are constructed implicitly from `T` (or explicitly
via `some(x)`) and absent values are written `none`.

**Niche representation:** Whenever `T` implements `Niche` (having at least one
invalid bit pattern), `?T` uses that pattern for `none` with **zero extra
bytes**:

- `?Int`: 8 bytes (niche `-2^63`)
- `?V` (any domain position or pool handle): 4 bytes (niche `UINT32_MAX`)
- `?0..=1e9` (any range type with a spare bit pattern): 4 bytes
- `?Bool`: 1 byte (niche `2`)
- `?E` (enum with fewer than 255 variants): 1 byte
- `?S` (struct whose first `Niche`-bearing field supplies a niche): `sizeof(S)`
- Otherwise (`?U64`, `?Float` — NaN is not assumed absent): `T` plus 1-byte
  flag, aligned naturally.

**Operations on `?T`:**

```cp
let x: Int = opt ?? 0                   // default fallback (short-circuiting)
let y: Int = opt!                       // unwrap (traps in debug if none)
let z: Int = opt?                       // early-return none from fn or do: block
let mapped = opt.map(v => v + 1)        // ?Int

if let d = dist[v]:
    print(d)

while let top = stack.last() if top.height <= a[x]:
    stack.pop()

match start[x]:
    none => g[x]
    some(t) => g[x] ^ (max(p - t, 0) % 2)
```

**Identity in `min` / `max`:** `min`, `max`, `min=`, and `max=` treat `none` as
the identity element (`min(none, x)` is `some(x)`, `min(some(a), some(b))` is
`some(min(a, b))`, `min(none, none)` is `none`). Ordering operators `<`, `<=`,
`>`, `>=` are a **compile error** on `?T`, so `none` can never silently sort
before or after real values. Equality (`== none`, `!= none`, `opt1 == opt2`) is
allowed.

### Tuples and Named Tuples

Anonymous positional tuples `(Int, V)` and named tuples `(lo: Int, hi: V)` are
structural types: two named tuples with identical field names and types in the
same order are the exact same type. Named tuples can be destructured either
positionally or by field name:

```cp
fn span(v: V) -> (lo: Int, hi: Int):
    (lo: tin[v].ord, hi: tout[v].ord)

let s = span(v)
let len = s.hi - s.lo
let (a, b) = span(v)
```

Tuples derive `Eq`, `Ord` (lexicographic), `Hash`, `Show`, `Read`, `Gen`, and
applicable representation traits automatically.

### Structs

Structs are nominal product types holding fields, default values, and methods
in one body:

```cp
struct Edge:
    u: V
    v: V
    w: 1..=1e9 = 1

    fn other(self, x: V) -> V = if x == self.u: self.v else: self.u
    @get fn heavy(self) -> Bool = self.w > 1e6
```

- **Generated constructors:** Every struct gets a named-field constructor
  (`Edge(u: a, v: b, w: 5)`, omitting defaulted fields as `Edge(u: a, v: b)`).
  Structs with 3 or fewer fields also support positional construction
  (`Edge(a, b, 5)`). Custom constructors are ordinary static functions
  (`Graph.undirected(edges)`).
- **Functional update (`with`):** `e with w: 7` produces a copy of `e` with `w`
  replaced (ideal for persistent trees and DP states).
- **Getters (`@get`):** A read-only zero-argument method annotated `@get` is
  accessed without parentheses (`e.heavy`, `s.len`).
- **Layout:** Fields are stored in declaration order (with standard C++
  alignment). No user-defined destructors exist; owned heap fields are freed
  automatically at end of scope.

### Enums

Enums are nominal sum types supporting exhaustive `match`. They have two forms:

```cp
// 1. Unit enum (automatically a Domain / Index type!)
enum Dir = Up | Down | Left | Right

// 2. Payload enum
enum Query:
    Add(l: V, r: V, delta: Int)
    Ask(i: V)
    Undo
```

- A unit enum (all variants have no payload) automatically implements `Domain`
  and `Index`, so you can index arrays directly by it: `var cnt: [Dir]Int = 0`,
  `for d in Dir: ...`.
- Payload enums use the smallest integer tag that fits (1 byte for up to 255
  variants) or reuse a payload niche when possible.
- Matching an enum must cover all variants (or end with `_ => ...`).

### Modular Arithmetic: `Mod<P>` and `DynMod`

`Mod<P>` represents integers modulo a compile-time constant `P`
(`2 <= P <= 2^31 - 1`), stored in **4 bytes** (`uint32_t`):

```cp
type M = Mod<1e9 + 7>
let ans: M = M(2) ** n - 1 - M.falling(n, a) / M.fact(a)
print(ans.int)                  // or print(ans) directly
```

- Integer literals (`0`, `1`, `2`, `-1`) convert implicitly **only** into
  `Mod<P>` when the expected type is `Mod<P>` (so `M(2) ** n - 1` works
  cleanly). Runtime `Int` values convert explicitly via `M(x)` (reducing via
  Euclidean `% P`), and `.int` extracts the canonical `0..<P` value.
- `+`, `-`, `*`, `**` (binary exponentiation in O(log e)) work for all `P`.
- `/` and `.inv()` require `P` to be prime (verified at compile time by the
  compiler!). For non-prime `P`, use `.try_inv() -> ?Mod<P>` (extended GCD).
- `Mod<P>` implements `Add`, `Sub`, `Mul`, `Div` (when prime), `Eq`, `Show`,
  `Read`, `Zero`, `Small`, and `Niche` (when `P < 2^32 - 1`), so `?Mod<P>` is
  still 4 bytes and `Mod<P>` plugs straight into `SegTree`, `Matrix`, `Poly`,
  and `Comb`.

When the modulus is read from input at runtime, use **`DynMod`** explicitly so
there is no hidden global modulus variable or silent overhead on static-mod
code:

```cp
let (n, mod_val): (1..=2e5, 2..=1e9) = read()
let ring = ModRing(mod_val)                 // explicit runtime modulus context (Barrett precomputed)
var x = ring.val(1)
for i in 1..=n: x = ring.mul(x, i)          // or x * ring.val(i) via DynMod carrying its ModRing handle
```

In the scoped form `with_mod(mod_val): ...` (which sets a scoped `ModRing`
token passed implicitly only inside that block, or explicitly as
`let m = ModRing(mod_val)`), `DynMod` stores a 4-byte `uint32_t` value and uses
the active Barrett multiplier from the `ModRing`. Nothing ever mutates a hidden
global modulus silently across functions.

---

## 5. Index Domains

Index domains are the centerpiece of CP's bug-prevention design. A domain is a
**nominal typed index space** representing a finite set of positions. Two
domains with the exact same numeric bounds are still distinct, incompatible
types:

```cp
let (n, m): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 1..=m

let w: [E]Int = read()
var deg: [V]Int = 0
let endpoints: [E](u: V, v: V) = read()

for e in E:
    deg[endpoints[e].u] += 1
    deg[endpoints[e].v] += 1

// deg[e]  --> compile error: `deg` is indexed by `V`, got `E`
```

### Representation and Properties

Every domain value is stored as its 0-based ordinal in **4 bytes** (`uint32_t`).
Because domains index contiguous arrays, a domain's declared maximum size must
not exceed `2^31 - 1` (larger coordinates remain `Int` until compressed with
`compress`). The domain's **label** (such as `1..=n`, `0..<n`, or `'a'..='z'`)
governs only `read()`, `print()`, `D(label)`, and `.label`; internal indexing
always uses the 0-based ordinal `.ord`.

- Indexing `[D]T` with a `v: D` is **never bounds-checked at runtime**, because
  no out-of-range value of type `D` can ever be constructed.
- Every linear domain `D` exposes: `D.len` (`Int`), `D.all` (`Range<D>`),
  `D.first` / `D.last` (`D` when `D` is statically known non-empty, `?D`
  otherwise), `D.bounds` (its boundary domain), `D.rev` (reverse traversal
  order).
- Every position `v: D` exposes: `v.ord` (`0..<D.len`), `v.label` (the label in
  the domain's declared range, e.g., `1..=n` or `'a'..='z'`), `v.before` and
  `v.after` (in `D.bounds`).

### Creating Positions Safely

There are four ways to construct a position in `D`, and each states its failure
policy explicitly:

```cp
let v = V(label)            // from label (1..=n): checked (traps in debug if out of range)
let u = V.at(ord)           // from 0-based ordinal: checked
let w = V.try_at(ord)       // -> ?V: returns none if out of range
if x in V:                  // flow refinement: `x` has type `V` inside this branch, zero check
    deg[x] += 1
```

Reading directly into `V` (`let edges: [E](V, V) = read()`) validates every
label at input time, so data enters the program already typed.

### Domain Arithmetic and Checking Rules

Stepping off the end of an array is one of the most frequent bugs in CP. Linear
domains therefore follow a strict **explicit boundary-policy rule**:

- **Position difference (`u - v -> Int`)** is always allowed for two positions
  in the same linear domain. It returns their signed ordinal distance
  (`u.ord - v.ord`), which can never go out of bounds.
- **Linear `v + k` and `v - k` are compile errors** on linear domains and cuts.
  A raw `+ k` on a linear domain hides whether the caller wanted `none`, a
  trap, clamping, or wrapping — violating the guiding test. The compiler error
  lists the exact alternatives:
  - **Fallible stepping (`-> ?D`):** `v.next`, `v.prev`, `v.offset(k: Int)`.
    Returns `none` if the result leaves the domain.
  - **Explicitly checked stepping (`-> D`):** `v.offset(k)!` (or `v.next!`,
    `v.prev!`). Traps in debug if out of range; the `!` makes the assumption
    visible at the call site.
  - **Clamped stepping (`-> D`):** `v.fwd(k: 0..)` and `v.back(k: 0..)`. Moves
    up to `k` steps forward or backward, clamping at `D.last` or `D.first`. It
    is total and can **never** trap or go out of bounds.
  - **Clipping an integer range (`-> Range<D>`):** `D.clip(lo..=hi)` or
    `D.clip(lo..<hi)` takes an `Int` range in the domain's label space (for
    cuts, `0..=n`) and intersects it with `D`, yielding a valid `Range<D>`
    (possibly empty).
- **Cyclic domains (`ring(n)`)** DO allow `v + k` and `v - k` (returning `V`),
  plus `v.next` and `v.prev` (returning `V`), because addition modulo `n` is a
  total closed operation that can never fail.

```cp
// Linear domain: explicit boundary policies
for v in V:
    if let u = v.next if a[u] < a[v]: drops += 1        // fallible: handles end cleanly
    let w = v.back(k) ..= v                             // clamped: never underflows!
    let far = v.offset(2)!                              // checked: invariant visible via !
```

### Cuts and Bounds (`I.bounds`)

Prefix sums, DP over prefixes, binary search insertion points, and interval
splitting all operate on the **gaps between elements**, not on elements
themselves. For any linear domain `I` of length `n`, `I.bounds` is a derived
domain of `n + 1` cuts (ordinals `0..=n`):

- `I.start` (= `Cut.first`) is the cut before the first element (`count == 0`).
- `I.end` (= `Cut.last`) is the cut after the last element (`count == n`).
- For `x: I`, `x.before: I.bounds` is the gap immediately left of `x` and
  `x.after: I.bounds` is the gap immediately right of `x`.
- For `b: I.bounds`:
  - `b.count: 0..=n` is the number of elements of `I` strictly before `b`
    (equal to `b.ord`).
  - `b.left: ?I` is the element immediately before `b` (`none` at `I.start`).
  - `b.right: ?I` is the element immediately after `b` (`none` at `I.end`).
  - `b.back(k)` and `b.fwd(k)` move up to `k` cuts left/right, clamping at
    `I.start` / `I.end`.
  - `b1 - b2 -> Int` is the number of elements between two cuts.
  - `I.between(c1, c2) -> Range<I>` forms the contiguous range of elements of
    `I` strictly between two cuts `c1, c2: I.bounds` (empty if `c1 >= c2`),
    keeping `Range<I>` (elements) distinct from `c1 ..< c2 : Range<I.bounds>`
    (a range of cuts themselves).

Because `I` and `I.bounds` are distinct types, indexing a prefix-sum array or
prefix-DP array with an element `x: I` is a **compile error**:

```cp
domain I = 1..=n
domain Cut = I.bounds

var pre: [Cut]Int = 0
for x in I: pre[x.after] = pre[x.before] + a[x]

// In Tourism (CCC 2019 S4): window of cuts from the previous day within k steps back
let prev_day = if d > 0: Cut.clip((d - 1) * k + 1 ..= d * k) else: Cut.first..=Cut.first
dp[b] = best.max(prev_day & (b.back(k) ..< b))
```

Notice that `b.back(k) ..< b` stays entirely inside `Cut` — never dropping to
raw `Int` and never trapping when `b.count < k`.

### Composite Domains: Products, Sums, and Sub-Domains

- **Product domains:** `domain Cell = (Row, Col)` creates a flat row-major
  product domain of size `Row.len * Col.len`. Arrays `[Row, Col]T` and
  `[Cell]T` have identical layout and support `g[r, c]` and `g[cell]`.
  Destructuring `for (r, c) in Cell:` yields `(Row, Col)`.
- **Sum (disjoint union) domains:** `domain Node = In + Out` creates a domain
  of size `In.len + Out.len`, ideal for split-vertex graphs, bipartite graphs,
  and 2-SAT (`domain Lit = True(V) + False(V)`). Variants construct and match
  cleanly:

```cp
domain Node = Left(L) + Right(R)
let u: Node = Node.Left(l)
match u:
    Left(l) => ...
    Right(r) => ...
```

- **Sub-domains (`Sub<D>`):** Created by slicing `a[r]`. A `Sub<D>` position
  converts implicitly to `D` (since every element of a slice is an element of
  the underlying domain) while indexing the slice itself with zero bounds
  checks.

### Standard Library Shape Domains and the `Domain` Trait

Contest shapes are NOT core syntax; they are ordinary standard library types
implementing the core `Domain` trait:

```cp
trait Domain:
    fn len() -> Int                         // domain size (may be runtime)
    fn ord(self) -> Int                     // 0..<len
    @trusted fn from_ord(i: Int) -> Self    // unchecked inverse for stdlib
```

Built-in standard library shape domains include:

- **`grid(r, c)`** (`domain Cell = grid(r, c)`): 1-based `(row, col)` product
  domain. Exposes `x.row`, `x.col`, `x.adj4` (in-bounds 4-neighbours),
  `x.adj8` (in-bounds 8-neighbours), `x.step(d: Dir4) -> ?Cell`, and
  `x.step(d: Dir8) -> ?Cell`. Reading a `Cell` reads two 1-based integers
  `(row, col)`. `grid0(r, c)` provides the 0-based variant.
- **`ring(n)`** (`domain V = ring(n)`): cyclic domain of size `n` (1-based
  labels by default; `ring0(n)` for 0-based). `v + k`, `v - k`, `v.next`, and
  `v.prev` wrap modulo `n` and return `V`. Cyclic ranges use explicit endpoint
  inclusivity: `V.arc(from: i, until: j)` (half-open) and
  `V.arc(from: i, through: j)` (closed).
- **`heap_tree(levels: h)`** (`domain Node = heap_tree(levels: h)`): complete
  binary tree on `1..=2**h - 1`. Exposes `v.children` (0 or 2 in-bounds child
  nodes), `v.left -> ?Node`, `v.right -> ?Node`, `v.parent -> ?Node`,
  `v.depth` (root has depth `1`), and `v.is_leaf`.
- **`mask(D)`** (`domain S = mask(D)` where `D.len <= 25` at runtime): domain
  of all `2 ** D.len` subsets of `D`, for bitmask DP (`var dp: [S]Int`).
  Exposes `s.contains(v)`, `s.with(v)`, `s.without(v)`, `s.submasks()`,
  `s.count`, `S.empty`, `S.full`.

```cp
// Grid domain (Orac and Game of Life)
domain Cell = grid(r, c)
let g: [Cell]Bit = read(digits)
let seeds = Cell.filter(x => x.adj4.any(y => g[y] == g[x]))

// Ring domain (One Fourth)
domain V = ring(n)
let js = V.arc(from: i + 2, through: i - 1)

// Heap-tree domain (Drazil Likes Heap)
domain Node = heap_tree(levels: h)
fn larger_child(v: Node) -> ?Node =
    v.children.filter(c => a[c] != none).max_by(c => a[c])
```

### Fresh (Existential) Domains

Functions may return a **fresh domain** whose identity is unique to that call
site. Two calls to `compress` or `scc` yield distinct, incompatible domain
types:

```cp
let (X, vals) = compress(xs)            // X is a fresh domain; vals: [X]Int
let (C, comp) = scc(g)                  // C is a fresh component domain; comp: [V]C
let (D, arr) = list.freeze()            // freeze a List<T> into [D]T
let s: [domain S]Char = read()          // inline fresh domain for string positions
```

Every `List<T>` also exposes a fresh read-only domain `xs.indices` valid as
long as `xs` is not structurally mutated (borrowing `xs` immutably).

### Escape Hatches

When an algorithm genuinely requires raw ordinal indexing (for example,
non-standard stride math in a tight DP kernel), CP provides explicit, greppable
escape hatches rather than silent conversions:

- `a.raw[i]` indexes `[D]T` by a 0-based `Int` (checked in debug; checked in
  judge unless proven or inside `unchecked:`).
- `D.at(i)` converts a 0-based `Int` ordinal to `D` (checked).
- `unchecked:` drops remaining runtime bounds checks inside a single block in
  judge/release builds while keeping them active in debug builds.

---

## 6. Arrays, Tables, Lists, Slices, and Views

### Domain Arrays (`[D]T` and `[A, B]T`)

`[D]T` is a fixed-size, contiguous heap array indexed exclusively by positions
of domain `D`. Multi-dimensional arrays `[A, B]T` are stored flat in row-major
order and indexed with `a[i, j]` (writing `a[i][j]` is a compile error, which
prevents confusing a 2D array with a jagged list of lists):

```cp
var dp: [V]Int = 0                      // all elements initialized to 0
var dist: [V]?Int = none                // all elements initialized to none
let g: [Row, Col]Char = read()
let ch = g[r, c]
let row_view = g.row(r)                 // zero-copy [Sub<Col>]Char / [Col]Char view
```

- **Zero-cost fill:** When `T: Zero` and the initializer is `0` / `false` /
  zero-valued, allocation uses zeroed memory (`calloc` / value-init) instead of
  a loop.
- **Count-indexed arrays:** When the index is genuinely a numeric value (like a
  frequency array), declare a range-indexed array `[0..=1e6]Int`. Indexing it
  with `a[v]` is unchecked when range analysis proves `a[v]` lies in `0..=1e6`,
  and checked otherwise.
- **Zero-cost rekeying:** When transferring a raw range array (such as a grader
  parameter `x0: [0..<n]1..=1e9`) into a nominal domain `Year = 0..<n` of the
  same length, `x0.rekey(Year)` moves the backing buffer in O(1) with zero
  copies (checking equal lengths in debug if not statically proven).

### Tables (`[D] i => expr`)

A table expression allocates and fills a `[D]T` in one expression:

```cp
let deg2 = [V] v => deg[v] * 2
let suf: [V]Int = [V.rev] v => a[v] + (v.next.map(u => suf[u]) ?? 0)
let dp: [V]Int = [V] v => max(a[v], (v.prev.map(u => dp[u]) ?? 0) + a[v])
```

- Specifying `[D.rev]` fills the array in reverse ordinal order while still
  returning a `[D]T`.
- **Self-referential DP tables:** Inside its own initializer, a table may read
  cells the compiler proves were already filled earlier in the fill order
  (`v.prev` or `v.offset(-k)` when filling forward; `v.next` or `v.offset(k)`
  when filling in reverse; or any index `u` statically proven `u < v` in a
  forward fill). Any unproven self-read is a compile error suggesting an
  explicit `var dp: [D]T` and a loop. There is no runtime "computed" flag.
- Comprehensions `[expr for v in V]` build a `List<T>` by default, or `[V]T`
  when the expected type is explicitly `[V]T` and there is no `if` filter.

### Growable Lists (`List<T>`)

`List<T>` is a growable contiguous array (lowering to `std::vector<T>`). Lists
are indexed by `Int` (0-based, bounds-checked unless proven by range analysis
or indexed via `xs.indices`):

```cp
var ops: List<Node> = []
ops.push(v)
let top = ops.last()                    // -> ?Node
let popped = ops.pop()                  // -> ?Node
let (D, arr) = ops.freeze()             // moves buffer into [D]Node with fresh domain D
```

List comprehensions build a `List<T>` directly:

```cp
let p = [pos - rank for (rank, pos) in s.positions('Y').enumerate()]
```

### Slices (`a[r]`)

Indexing an array `[D]T` with a `Range<D>` produces a **zero-copy slice**
borrowing `a`:

```cp
let part = a[l..=r]                     // read-only slice borrowing a
let idx = argmax(part)                  // returns Sub<D> (or ?Sub<D>), convertible to D
print(other_arr[idx])                   // valid! positions keep their original domain D
a[l..=r].sort()                         // in-place mutable borrow of that slice
let owned = a[l..=r].copy()             // explicit allocation when an owned copy is needed
```

**Positions retain their original domain identity:** A slice's index domain is
`Sub<D>`, whose values convert implicitly to `D`. Consequently, `argmax(a[l..=r])`
returns the position in `D` (not a 0-based offset relative to `l`), completely
eliminating C++'s `max_element(a + l, a + r) - a` versus `- (a + l)` bug.
Slicing a slice (`a[s][t]`) likewise takes a `Range<D>` in the original domain,
checked to lie within `s`.

### Lazy Views and Generators (`Iter<T>`)

A view (`a.map(f)`, `V.filter(p)`, `runs(a)`, `windows(V, k)`) stores nothing
and allocates nothing. All custom lazy iteration is unified under **generators**
— functions declared with `yields T` that emit elements via `yield` and
delegate via `yield from`:

```cp
pub fn submasks(m: U64) yields U64:
    var s = m
    loop:
        yield s
        if s == 0: break
        s = (s - 1) & m

pub fn preorder(t: RootedTree<V>, v: V) yields V:
    yield v
    for c in t.children(v): yield from preorder(t, c)
```

**Rules and lowering guarantees:**

- **Push lowering (default):** When consumed by a `for` loop, comprehension,
  reduction, or single-stream adapter chain (`filter`, `map`, `take`, `skip`,
  `enumerate`, `flat_map`, `chain`), the compiler inlines the consumer's loop
  body directly at every `yield`. `break`, `continue`, and `return` inside the
  consumer exit the generator cleanly, even across recursion. Zero allocations,
  zero iterator objects, zero indirect calls.
- **Pull lowering (for `zip` and explicit suspension):** Non-recursive
  generators may also be consumed in parallel (e.g., `zip(g1, g2)`), where the
  compiler lowers them to a flat, stack-allocated state machine (no heap
  allocation). Using a **recursive** generator in a pull position (`zip` of two
  recursive traversals) is a compile error suggesting collecting into a `List`
  first — preventing hidden heap stacks.
- **Single consumption:** An `Iter<T>` parameter may be iterated at most once;
  iterating twice is a compile error suggesting `List(xs)` so an expensive
  producer is never re-run silently.
- **Explicit stored iterators (`Iterator<T>`):** When you genuinely need a
  resumable iterator stored in a variable or struct (such as a k-way merge
  heap), implement the `Iterator<T>` trait (`fn next(mut self) -> ?T`) on a
  regular struct so its state and byte size are completely explicit.

---

## 7. Ranges

CP standardizes on two unambiguous bounded range forms and one lower-bounded
form:

- `a ..< b` — **half-open** (includes `a`, excludes `b`).
- `a ..= b` — **closed** (includes both `a` and `b`).
- `a ..` — **lower-bounded** (used in range types like `1..` and open slices).

Bare `a..b` is intentionally disallowed so half-open vs inclusive endpoints are
always visually explicit at every call site.

### `Range<D>` and `Range1<D>`

A domain range `Range<D>` is stored as two cuts in `D.bounds` (**8 bytes**),
representing a possibly empty contiguous run of positions. `Range1<D>` is its
non-empty subtype (guaranteed `len >= 1`), converting implicitly to `Range<D>`.

- On **`Range<D>`**:
  - `r.len -> Int` (0 when empty)
  - `r.is_empty -> Bool`
  - `r.first -> ?D`, `r.last -> ?D`, `r.mid -> ?D` (lower median)
  - `r.before -> D.bounds`, `r.after -> D.bounds`
  - `r.contains(v: D) -> Bool`
  - `r.nonempty() -> ?Range1<D>`
  - `r & s -> Range<D>` (intersection; empty if disjoint)
  - `r.clipped(s: Range<D>) -> Range<D>` (alias for `r & s`, also accepts
    an `Int` index range on sequence/arc ranges via `js.clipped(t - 1 ..= t)`)
- On **`Range1<D>`**:
  - `r.first -> D`, `r.last -> D`, `r.mid -> D` (non-optional! No `!` needed.)
  - Yielded by views guaranteed non-empty: `runs`, `windows`, `chunks`, and
    `longest_window` (which returns `?Range1<D>`).

```cp
// Longest Y (ABC 229 G): subranges + .len eliminate all +1/-1 arithmetic
fn cost(w: Range1<P>) -> Int:
    let m = w.mid
    let lo = w.first ..< m
    let hi = m ..= w.last
    (p[m] * lo.len - pre.sum(lo)) + (pre.sum(hi) - p[m] * hi.len)
```

### Empty Ranges, Steps, and Reversal

- A **computed** range whose start exceeds its end (such as `l..=r` when
  `l > r`) is an empty range (`len == 0`), matching standard loop boundary
  behavior. A **literal** inverted range (such as `5..=3`) is a compile error;
  use `(3..=5).rev` to iterate downward.
- Steps use `by k` (`0..<n by 2`, `k > 0`); reversal uses `.rev`
  (`(l..=r).rev`, `V.rev`).

---

## 8. Control Flow and Patterns

### Conditionals

`if` / `elif` / `else` is an expression when all branches produce a compatible
type. Inline conditionals use the single canonical syntax `if cond: a else: b`
(postfix `a if cond else b` is disallowed to avoid grammar ambiguity with loop
guards):

```cp
fn inv(r: 0..=n) -> Int =
    if r <= 1: 0 else: ask("? 1 {r}", as: Int)

let label = if ok: "YES" else: "NO"
```

`if let pat = expr:` binds optional or enum values; optional guards use `if`:

```cp
if let u = v.next if a[u] < a[v]:
    drops += 1
```

### Pattern Matching (`match`)

`match` is an expression and is checked for exhaustiveness. Arms use `=>`:

```cp
let color = match start[x]:
    none => g[x]
    some(t) => g[x] ^ (max(p - t, 0) % 2)

match query:
    Add(l, r, d) if d != 0 => seg.update(l..=r, d)
    Add(_, _, _) => ()
    Ask(i) => print(seg[i])
    Undo => history.undo()
```

Supported patterns: literals, integer/char ranges (`'0'..='9'`, `1..=3`),
enum variants (`some(x)`, `Add(l, r, d)`), tuples and structs
(`Edge(u, v, w: 1)`), alternation (`Up | Down`), wildcard `_`, binding with
`if guard`.

### Loops

CP provides five clean loop forms:

```cp
// 1. For-in with optional filter guard
for v in Node if v.depth <= g:
    ...

// 2. Repeat n times (no dummy loop variable)
repeat q:
    let (x, p): (Cell, 1..=1e18) = read()
    ...

// 3. While
while lo < hi:
    ...

// 4. While-let with optional guard (ideal for monotone stacks & heaps)
while let top = stack.last() if top.height <= a[x]:
    best.update(top.cuts, a[x] - top.height)
    cuts = top.cuts.first ..= cuts.last
    stack.pop()

// 5. Infinite loop (exited via break or return)
loop:
    ...
```

Loops may carry labels (`outer: for i in V:`) targeted by `break outer` or
`continue outer`.

### Compound Assignment

In addition to `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`,
CP provides **`min=`** and **`max=`** natively:

```cp
best min= abs(total - piece(i, j))
dp[v] max= cand                         // also works when dp[v] is ?Int (none is identity)
```

### `defer` and Scoped Rollback (`undo`)

- **`defer stmt`** runs `stmt` on every exit from its enclosing block (normal
  fallthrough, `break`, `continue`, `return`, `?`), in LIFO order. It lowers
  statically onto each exit edge with zero runtime registration overhead.
- **`undo x, y: ...`** snapshots listed state before the block and restores it
  on every exit from the block. Scalars (`Small`) are saved on the stack;
  data structures must implement the `Rollback` trait (`mark()` / `undo_to()`).
  Because state must be listed explicitly, journaling never happens behind the
  programmer's back:

```cp
undo dsu, best:
    dsu.union(e.u, e.v)
    best max= dsu.size(e.u)
    dfs(child)
// dsu and best are restored here
```

---

## 9. Mutation, Ownership, and Borrowing

CP uses a **simple value-ownership model with no lifetime annotations, no raw
pointers, and no stored references**. It prevents accidental O(n) copies,
use-after-free, and aliased mutation bugs without Rust's lifetime ceremony.

### Core Ownership Rules

1. **Variables own their values.** `let a: [V]Int = read()` owns the array and
   frees it at end of scope.
2. **Function parameters borrow by default.** `fn f(a: [V]Int)` borrows `a`
   read-only with zero copy. `fn f(mut a: [V]Int)` borrows `a` mutably, and
   the call site writes `f(mut a)`. Method receivers (`self` / `mut self`) omit
   `mut` at the call site.
3. **Copy vs move is explicit for big values.** Types implementing `Small`
   (at most 16 bytes, no owned heap buffers: `Int`, `Float`, `Bool`, `Mod<P>`,
   domain positions, ranges, small structs) copy implicitly. Big values
   (`[D]T`, `List<T>`, `Str`, `SegTree`, `Graph`, structs containing them)
   never copy implicitly:
   - Using a big value for the last time **moves** it implicitly in O(1).
   - Keeping the original alive while duplicating it requires **`.copy()`**.
4. **No stored references.** A struct or container can never hold a reference
   or borrow into another variable. Views (`a[r]`, `a.map(f)`) live only as
   local borrows or function arguments, never inside structs or returned from
   functions.
5. **Exclusive mutation.** While a place is borrowed mutably, nothing else may
   read or write it. Disjoint struct fields borrow independently
   (`f(mut s.left, s.right)` is valid). Distinct array indices do not split
   automatically (`f(mut a[i], a[j])` is a compile error suggesting
   `swap(mut a, i, j)` or `let (x, y) = a.pair_mut(i, j)` which checks `i != j`
   once).
6. **Two-phase borrows.** `xs.push(xs.len)` is legal because the mutable borrow
   of `xs` activates only after arguments finish evaluating.
7. **Moves out of containers.** No partial moves out of arrays or lists; use
   `swap(mut a, i, j)` or `take(mut a[i])` (which moves the value out and
   leaves the `Zero`/`none` default behind).

```cp
var dsu = Dsu<V>()
fn merge(mut d: Dsu<V>, u: V, v: V): d.union(u, v)

merge(mut dsu, a, b)                    // explicit mut at call site
let backup = dsu.copy()                 // explicit O(n) copy
consume(dsu)                            // last use: O(1) move
```

### Inline Closures

Higher-order stdlib functions (such as `sort`, `bisect`, `bfs`, `sliding`) take
their closure parameters as non-escaping inline callbacks. Because the compiler
inlines them directly into the caller, they may borrow enclosing locals cleanly
without lifetime parameters. Two sibling closures in the same call (such as
`add` and `remove` in `sliding`) cannot both mutably capture the same outer
`var`; instead, `sliding` threads the state value explicitly through `add` and
`remove` (moving large state in place at zero cost).

### Pools and Handles (Identity Without Pointers)

Node-based structures — treaps, tries, persistent segment trees, suffix
automata, Aho-Corasick, link-cut trees — use **pools and typed 4-byte handles**
instead of pointers:

```cp
struct Node:
    key: Int
    pri: U32
    l: ?Nodes = none
    r: ?Nodes = none

pool Nodes: Node                        // declares pool type and nominal 4-byte handle `Nodes`
var nodes = Nodes.new(cap: 4e6)         // preallocated flat arena
let h: Nodes = nodes.add(Node(key: k, pri: rng.u32()))
nodes[h].l = other
let h2 = nodes.add(nodes[h] with l: new_left)   // persistent node copy in O(1)
```

- Handles are nominal (just like domains): a `Nodes` handle cannot index a
  different pool or a `[V]` array.
- Handles can only be created by `add`, so indexing is never out of bounds and
  `?Nodes` uses the 4-byte `UINT32_MAX` niche.
- Default pools are arenas (never free, maximum speed). `Nodes.new(free: true)`
  adds a free list with `nodes.remove(h)`; debug builds detect use-after-free
  via generation tags.

---

## 10. Abstraction: Traits, Operation Values, and Generics

CP splits polymorphism cleanly along one sharp boundary: **traits** for
behavior that a type has *one canonical instance* of, and **operation values**
for behavior that a type can have *many instances* of.

### Traits (One Canonical Instance)

Traits define canonical operations (`Eq`, `Ord`, `Hash`, `Show`, `Read`, `Add`,
`Sub`, `Mul`, `Div`, `Domain`, `Rollback`, `Iterator`):

```cp
trait Ord: Eq
    fn lt(self, other: Self) -> Bool
    fn le(self, other: Self) -> Bool = not other.lt(self)
    law transitive(a: Self, b: Self, c: Self) =
        not (a.lt(b) and b.lt(c)) or a.lt(c)

impl Ord for Frac:
    fn lt(self, other: Frac) -> Bool = self.p * other.q < other.p * self.q
```

- **Coherence:** `impl Trait for Type` may appear only in the module defining
  `Trait` or `Type`. Two modules can never disagree on `Ord` for `Frac`.
- **Static dispatch only:** All trait calls resolve at compile time and inline.
- **Default derivation:** Structs and enums automatically derive `Eq`, `Ord`
  (field order), `Hash`, `Show`, `Read`, `Gen`, `Shrink`, and all applicable
  representation traits whenever all fields support them. Write an explicit
  `impl` only to override.

### Representation Traits

The compiler derives eight zero-runtime-cost representation traits used inside
`if const` branches in the stdlib:

- `Niche`: has a spare bit pattern for zero-overhead `?T`.
- `Index`: maps bijectively onto `0..<K` (domains, unit enums, `Bool`, tuples
  of `Index` types).
- `Small`: at most 16 bytes, no heap pointers; copied implicitly.
- `Relocate`: safely movable via `memcpy` (used by `List` growth and sort).
- `Zero`: all-zero bytes is a valid value (enables zero-initialized buffers).
- `Bits`: fits in `T.bits` bits (used by explicit `Packed<T>`).
- `OrdKey`: order-preserving `U64` key (enables radix sort).
- `Bounded`: exposes `T.MIN` and `T.MAX`.

These traits **never** change container complexity or silently swap data
structures; they only guide lossless low-level lowering (such as `calloc` vs a
fill loop, or `memcpy` vs move loops).

### Operation and Ordering Values (Many Instances per Type)

Algebraic operations (sum, min, max, xor, gcd, matrix multiply, affine
updates) and custom orderings (min-heap, max-heap, sort by key) are **ordinary
structs of functions and constants**:

```cp
pub struct Semigroup<T>(op: (T, T) -> T):
    law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))

pub struct Monoid<T>(id: T, op: (T, T) -> T):
    law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))
    law ident(a: T) = op(id, a) == a and op(a, id) == a

pub struct Group<T>(id: T, op: (T, T) -> T, inv: T -> T):
    law inverse(a: T) = op(a, inv(a)) == id

pub struct Idempotent<T>(op: (T, T) -> T):
    law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))
    law idem(a: T) = op(a, a) == a

pub struct Action<T, F>(apply: (F, T, Int) -> T, compose: (F, F) -> F, id: F):
    law act_id(x: T, len: 1..=100) = apply(id, x, len) == x
    law act_comp(f: F, g: F, x: T, len: 1..=100) =
        apply(compose(f, g), x, len) == apply(f, apply(g, x, len), len)

pub struct Order<T>(lt: (T, T) -> Bool)
```

Notice `Action.apply` takes the segment length `len: Int` so both length-free
actions (range-add on `max`) and length-dependent actions (range-add or
range-assign on `sum`) share one uniform type.

**Structural subsumption:** A `Group<T>` passes anywhere a `Monoid<T>` or
`Semigroup<T>` is expected; a `Monoid<T>` passes where a `Semigroup<T>` is
expected. A `Fenwick` requires `Group<T>` (rejecting `max` at compile time!),
a `SparseTable` requires `Idempotent<T>` (rejecting `sum` at compile time!),
and a `SegTree` requires `Monoid<T>`.

**Zero-cost specialization (compile-time function identity):** Every function
and lambda has a unique compile-time identity carried by any struct field that
holds it. Generic structures and functions monomorphize per operation identity:
`SegTree<V>(max, update: add)` compiles with `max` and `add` fully inlined,
zero function pointers, zero virtual dispatch. If two control-flow branches
return operation values with different function identities (`if k: max else:
min`), that is a compile error unless the receiving type explicitly opts into
an indirect call via `dyn (T, T) -> T`.

**Laws:** `law` clauses on traits and operation structs are property-tested by
`cp fntest` and sampled on real runtime values in debug builds, catching
broken custom segment-tree merges (like non-associative max-subarray combines)
immediately. Laws are completely stripped in judge builds.

### Generics and Monomorphization

Generic parameters support types (`T`), domains (`D`, constrained to `Domain`),
and compile-time scalar constants (`const N: Int`):

```cp
fn count_true<D>(a: [D]Bool) -> Int = count(v for v in D if a[v])
fn max_of(xs: List<T>) -> ?T where T: Ord = ...      // single capital letter: implicit <T>
struct Matrix<const N: Int, T>(data: [0..<N, 0..<N]T)
```

- **Definition-site checking:** Generic bodies are type-checked once against
  their `where` bounds — never producing C++ template-instantiation error
  walls.
- **Layout-sharing monomorphization:** All domains (`V`, `E`, `Cell`, `Year`)
  have identical 4-byte `uint32_t` runtime representation, so `List<V>` and
  `List<E>`, or `SegTree<V, Int>(max)` and `SegTree<Cut, Int>(max)`, share a
  **single emitted C++ instantiation**. This keeps submissions comfortably
  under the 64 KB source limit even when a solution declares many domains.

### `if const`, Config Structs, `Buffer<T>`, and `@trusted`

- **`if const cond:`** evaluates a compile-time condition (`T: Zero`,
  `const N <= 64`, `cfg.local`) and eliminates untaken branches before C++
  emission. Both branches are type-checked.
- **`const struct`** bundles compile-time feature flags so one stdlib type
  (such as `RootedTree`) pays zero memory and zero emitted bytes for features
  not requested.
- **`Buffer<T>` and `@trusted`:** The standard library itself is written in CP
  on top of one unsafe primitive: `Buffer<T>` (raw uninitialized capacity
  backing `List`, `Deque`, `Heap`, `HashMap`, `Pool`). Its raw methods
  (`init`, `take`, `move`) are callable only inside greppable `@trusted:`
  blocks. Debug builds maintain shadow initialization bits per slot and trap
  any read-before-init or double-init at the exact source line.

---

## 11. Program Structure

### The File Is a Function Body

A program file contains top-level statements and declarations and executes top
to bottom as an implicit `main` body — no `fn main()` boilerplate is ever
needed:

```cp
let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]Int = read()

if n == 1:
    print(a[V.first])
    return

fn solve() -> Int = ...
print(solve())
```

Declarations (`fn`, `struct`, `enum`, `domain`, `type`, `const`) may appear
anywhere in the file; calling a function before a top-level `let` or `var` it
captures has been initialized is a compile error. `return` at the top level
flushes output and ends the program (or ends the current test case under
`@multitest`).

### Multitest Programs (`@multitest` and `static let`)

Adding `@multitest` at the top of a file wraps the entire file body in a test
loop. Nothing inside the solution changes between single-test and multitest:

```cp
@multitest(sum: n <= 2e5)
static let comb = Comb<Mod<998_244_353>>(upto: 2e5)

let (n, k): (1..=2e5, 1..=n) = read()
domain I = 1..=n
let a: [I]Int = read()
// all locals, vars, and domains are fresh per test; nothing can leak!
```

Supported `@multitest` forms (all drive the exact same file body):

- `@multitest`: reads `t: 1..` first, runs the file body `t` times.
- `@multitest(case: tc)`: reads `t`, binds `tc: 1..=t` in scope for
  `"Case #{tc}: {ans}"` output (Code Jam / Hacker Cup style).
- `@multitest(until_eof)`: repeats the file body until standard input reaches
  EOF.
- `@multitest(until: cond)`: repeats until a sentinel condition holds right
  after the leading read (e.g., `let n: 0..=1e5 = read(); if n == 0: return`
  inside `@multitest(until_eof)`, or `@multitest(until: n == 0)`).
- `@multitest(count: q)` / `@multitest(sum: n <= 2e5)`: optional metadata used
  by stress generators and time/memory budget diagnostics.

`static let` runs once on first entry and stays immutable across all tests.
`static var` does not exist.

### Interactive Programs (`@interactive`)

Adding `@interactive` enables interactive I/O mode:

```cp
@multitest @interactive
let n: 4..=1e9 = read()

fn inv(r: 0..=n) -> Int =
    if r <= 1: 0 else: ask("? 1 {r}", as: Int)

let all = inv(n)
let k = bisect(1..=n, x => inv(x) == all)!
let j = k - (all - inv(k - 1))
let i = j - 1 - (inv(j - 1) - inv(j - 2))
answer("! {i} {j} {k}")
```

- **`ask(line, as: T)`** prints `line`, flushes stdout immediately, and reads a
  typed reply of type `T`.
- **`answer(line)`** prints `line` and flushes stdout immediately.
- **Clean wrong-answer exit:** In `@interactive` mode, whenever the judge
  replies with `-1` (configurable via `@interactive(fail: -1)` or
  `@interactive(fail: none)` when `-1` is a valid reply), `ask` exits cleanly
  with status 0 instead of triggering TLE/idle-limit exceeded.
- **Local interactive testing (`protocol` and `@interactor`):** For local
  stress-testing and adaptive opponent testing without external pipe scripts,
  a file may define a `protocol` and `@interactor` struct (stripped completely
  from judge C++ output); see Section 16 (`std.interactive`).

### Grader-Function Programs (`@grader`)

Online judges like IOI, APIO, and DMOJ interactive/grader tasks link the
submission against a C++ grader rather than calling `main()`. CP supports both
stateless and stateful grader problems cleanly under `@grader`:

**1. Stateless grader** (one or more exported functions, optionally calling
grader-provided `extern fn`s):

```cp
@grader
extern fn press(p: Str) -> Int

export fn guess_sequence(n: 1..=2000) -> Str:
    ...
```

**2. Stateful init-then-query grader** (`@grader(init: fn ...)`): When the
grader calls an initialization function first (which determines domain sizes)
and later calls query/update functions, the file's top-level statements **are**
the body of that `init` function! All domains, `let`s, and `var`s declared at
top level become the initialized grader state, visible to every `export fn`
(read-only) and `export mut fn` (read-write) declared in the file:

```cp
@grader(init: fn init(n: 1..=5e5, x0: [0..<n]1..=1e9, y0: [0..<n]1..=1e9) -> Int)
type M = Mod<1e9 + 7>

domain Year = 0..<n
var x: [Year]1..=1e9 = x0.rekey(Year)
var y: [Year]1..=1e9 = y0.rekey(Year)
var big: SortedSet<Year> = Year.filter(i => x[i] > 1).collect()
var prod = SegTree<Year>(M.mul, init: x.map(M))
var top = SegTree<Year>(max, init: y)

fn answer() -> Int:
    var rel = 0
    var end = Year.end
    for s in big.rev().chain([Year.first]):
        rel max= top.max(Year.between(s.before, end))
        if s == Year.first or rel * x[s] > 1e9:
            return (prod.fold(Year.first ..= s) * rel).int
        rel *= x[s]
        end = s.before

export mut fn update_x(i: Year, v: 1..=1e9) -> Int:
    big.set(i, v > 1); x[i] = v; prod[i] = M(v)
    answer()

export mut fn update_y(i: Year, v: 1..=1e9) -> Int:
    y[i] = v; top[i] = v
    answer()

return answer()
```

- **No uninitialized globals, no late-bound domains:** `Year`, `x`, `y`, `big`,
  `prod`, and `top` are initialized in order when the grader calls `init`, and
  never exist in an uninitialized state.
- **Name and ABI mapping:** `@grader` maps snake_case exports to camelCase if
  specified (`@grader(naming: camel)` maps `update_x` to `updateX`, or
  explicit `@export(name: "updateX")`), and automatically adapts C++ grader
  types (`int` / `int[]` / `std::vector<int>` according to the grader header
  profile, checking domain ranges on entry in debug builds).

### Library Files and Imports

A file with **no top-level statements** (only `fn`, `struct`, `enum`, `trait`,
`impl`, `domain`, `type`, `const`, `static let`, and `@test fn`) is a **library
file**. `use` may import only library files:

```cp
use std.segtree
use std.graph
use lib.treap as tp
```

- Module imports form a strict DAG (cyclic imports are a compile error).
- Only items marked `pub` are visible outside a library file.
- Every imported module is **inlined and tree-shaken** into the single output
  C++17 file: unused functions, unused struct methods, and `@test` functions
  cost zero bytes.

### Build Configuration and `@memo`

- `cfg` is a compile-time record (`cfg.local`, `cfg.debug`, `cfg.judge`, plus
  user `-D` flags). `if const cfg.local: ...`, `debug(x, y)` (stderr trace with
  file/line, stripped in judge builds), and `@local fn` (stripped in judge
  builds) keep debugging zero-cost on the judge.
- **`@memo`** caches a recursive function's results. It forbids capturing any
  enclosing `var` and forbids I/O. When every parameter is a domain or bounded
  range whose product fits the memory budget, `@memo` allocates a flat dense
  table `[P1, P2]?R` (using `R`'s niche so no extra "visited" array is
  needed); otherwise it is a compile error unless you explicitly opt into
  `@memo(hash)` (no hidden hash-table fallback!). `@memo(into: dp)` exposes the
  backing table under name `dp`.

---

## 12. Typed Input and Output

### Typed Input (`read`)

`read()` is a generic function driven by its expected return type (`Read`
trait). The type you declare IS the format you parse — no format strings, no
boilerplate read loops, and no raw-integer indexing mistakes:

```cp
let (n, m): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 1..=m

let a: [V]0..=1e9 = read()                      // reads n integers, range-checked
let es: [E](u: V, v: V, w: 1..=1e9) = read()    // reads m triples; u, v checked in V
let g: [Cell]Bit = read(digits)                 // r lines of c '0'/'1' characters
let chars: [Row, Col]Char = read()              // r lines of c non-whitespace chars
let edges: List<(V, V)> = read(n - 1)           // explicit count
let token: Str = read()                         // next whitespace-delimited token
let line: Str = read_line()                     // full line up to '\n'
let all_qs: List<Query> = read_until_eof()      // read until EOF
```

Key rules for `read()`:

- **Domain validation on arrival:** Reading into `V` reads a label in `V`'s
  declared label space (`1..=n`, `0..<n`, or `'a'..='z'`) and converts it to
  `V`'s 0-based ordinal. In debug builds, an out-of-range label traps with the
  exact input line and token column. Reading into `?V` parses `-1` as `none` by
  default (customizable via `@read(none: 0)`).
- **`read(digits)`:** Reads dense character grids without spaces (`'0'`/`'1'`
  for `Bit`, `'0'`..`'9'` for `0..=9`).
- **Refinements and structured input types:** `where` clauses validate
  cross-field invariants on arrival, and stdlib constraint types check
  structural invariants in debug builds while erasing to zero overhead in
  judge builds:
  - `Perm<V>`: permutation of `V` (checks distinctness + coverage in debug;
    stores `[V]V`).
  - `Tree<V>`: `n - 1` edges forming a connected acyclic graph (checked via DSU
    in debug).
  - `Distinct<[D]T>`, `Sorted<[D]T>`: checks uniqueness / monotonicity in debug.
- **Custom input formats:** Any user struct or enum can override `impl Read` to
  parse arbitrary custom formats (e.g., query type tags `1 l r` vs `2 i x`):

```cp
enum Query:
    Add(l: V, r: V, x: 1..=1e9)
    Ask(i: V)

impl Read for Query:
    fn read(mut r: Reader) -> Query:
        match r.read<1..=2>():
            1 => Query.Add(r.read(), r.read(), r.read())
            2 => Query.Ask(r.read())
```

### Typed Output (`print`)

`print(...)` outputs any `Show`-implementing value followed by a newline,
separating multiple arguments with single spaces. Lists and 1D domain arrays
print space-separated with **no trailing space** (eliminating C++'s
`" \n"[i + 1 == n]` trick); 2D grids print one row per line:

```cp
print(ans)                                      // scalar + newline
print(ops)                                      // "3 1 5 2\n" (space-separated)
print(ans_arr, layout: lines)                   // one element per line
print(ans_arr, sep: ", ")                       // custom separator
print("Case #{tc}: {ans}")                      // string interpolation
print(ratio, digits: 9)                         // fixed 9 decimal digits
print("{ratio:.9}")                             // identical via format spec
```

Printing `?T` directly prints the value or `none` (a lint suggests `?? -1` or
`?? "NO"` so contest output never accidentally prints `none`). Output is
buffered in a large output buffer and flushed automatically at exit — or
immediately on every `ask` / `answer` in `@interactive` mode.

---

## 13. Testing Hooks and Stress Testing

CP treats local verification as a first-class workflow while guaranteeing that
**zero bytes of testing code ever reach the judge C++ output**.

### `@brute` (Reference Solutions)

`@brute(target)` pairs a slow, obviously correct reference function with a
fast function (or with the whole program). An optional `when:` guard restricts
the brute to small inputs:

```cp
fn solve(a: [V]Int) -> Int:
    ...

@brute(solve, when: V.len <= 10)
fn solve_slow(a: [V]Int) -> Int =
    max(score(p) for p in perms(V))
```

- When `solve` is a part of the solution (not the whole program), `cp stress`
  tests `solve` directly on inputs generated from the file's `read()`
  declarations, isolates the exact failing function, and shrinks the failing
  input automatically.
- `@brute` functions are stripped completely during `cp build` / judge
  emission.

### Generators Derived from Input Types (`Gen` and `Shrink`)

Every readable input type automatically derives `Gen` (random test generation)
and `Shrink` (minimal counterexample reduction):

- `let (n, k): (1..=2e5, 1..=n) = read()` generates `n` across edge-biased
  distributions (tiny `n = 1, 2, 3`, boundary values, random sizes) and then
  generates `k` conditioned on `1..=n`!
- `where` clauses (`let (l, r): (V, V) = read() where l <= r`) are enforced by
  generation and preserved by shrinking (the shrinker never produces an invalid
  input).
- Structured types (`Perm<V>`, `Tree<V>`, `Graph<V>`, `Distinct`, `Sorted`)
  generate contest-adversarial shapes automatically: paths, stars, caterpillars,
  bipartite graphs, all-equal arrays, strictly sorted / reversed arrays, and
  extremal weights, and shrink structurally (pruning leaves, removing queries,
  lowering values).
- Under `@multitest(sum: n <= 2e5)`, `cp stress` respects the total-sum budget
  across generated tests.
- Custom generators can override `impl Gen for T` when a problem requires a
  planted structure (such as guaranteeing a valid solution exists).

### `@checker`, `@sample`, `@test`, and Laws

- **`@checker`** validates problems with multiple valid answers. Its parameter
  type defines how the solution's output is parsed, and it can read all
  top-level input `let`s:

```cp
@checker(output: ?[V]Color)
fn valid(c: [V]Color) -> Bool =
    all(c[e.u] != c[e.v] for e in edges)
```

  When paired with a `@brute`, `cp stress` verifies both existence (whether the
  brute found a solution vs `none`) and validity via `valid`.
- **`@sample(input_str, expected_str)`** embeds inline sample tests run by
  `cp test` alongside sample files in `<name>.samples/`.
- **`@test fn name():`** defines unit tests (especially in library files) run
  by `cp fntest`.
- **`law` clauses** on traits and operation values (`Monoid`, `Group`,
  `Action`, `Order`) are automatically fuzz-tested by `cp fntest` and shrunk
  to minimal counterexamples.

---

## 14. Safety Model: Static Proofs, Debug Traps, and Release

CP distinguishes three build profiles — **debug** (`cp run` / `cp build
--debug`), **judge** (`cp build`, the default for submissions), and **release**
(`cp build --release`) — and never silently hides which checks remain active.

### Tier 1: Proven Statically (Zero Runtime Cost Everywhere)

The compiler eliminates checks whenever static types, flow facts, or range
analysis prove safety:

- Indexing `[D]T` with a `v: D` or `Sub<D>`: **always 0 checks** in all builds.
- Flow-refined indices (`if x in V:`): **0 checks** inside the branch.
- Count-indexed arrays `[0..=M]T` indexed by a value whose range fits in
  `0..=M`: **0 checks**.
- Arithmetic whose propagated interval stays within `[Int.MIN, Int.MAX]`:
  **0 overflow checks**.
- Division `/` and remainder `%` on operands proven non-negative: plain C++ `/`
  and `%` with **0 adjustment instructions**.
- Non-empty reductions (`min`/`max`/`argmax` on non-empty domains or
  `Range1<D>`) and `Range1<D>.first`/`.last`/`.mid`: **0 emptiness checks**.

### Tier 2: Dynamic Checks in Debug vs Judge vs Release

Every remaining runtime condition has an explicit, documented policy:

- **Domain creation (`V(x)`, `V.at(i)`), `List` indexing (`xs[i]`), and
  `a.raw[i]`:** Checked in **debug** (traps with value, domain/length, and
  source line). Kept in **judge** unless proven statically or inside an
  `unchecked:` block. Dropped in **--release**.
- **Unproven integer overflow and narrowing stores:** Trapped in **debug**
  (producing `-2^63` on `Int` is also trapped as overflow so the `?Int` niche
  is never corrupted). Dropped in **judge** and **--release** (because proven
  definite overflows are already compile errors, and `cp build --checks` lists
  every remaining unproven arithmetic site).
- **Division / remainder by zero:** Trapped in **debug**; unchecked in judge
  and `--release`.
- **Optional unwrap (`opt!`) and `offset(k)!`:** Trapped in **debug**;
  unchecked in judge and `--release`.
- **Input range and `where` validation (`read()`):** Checked in **debug** (with
  exact line/column of offending token); trusted in judge and `--release` (so
  fast I/O stays at full speed).
- **`assert cond`:** Checked in **debug** AND kept in **judge** (a clean
  runtime-error signal on a judge is far easier to diagnose than a silent WA);
  dropped only in **--release**.
- **Contracts (`requires`, `ensures`, `invariant`):** Checked in **debug** and
  used to feed relational facts to range analysis warnings; dropped in judge
  and `--release` (unproven user contracts are never trusted to delete safety
  checks in judge builds).
- **Structural checks:** `Buffer<T>` slot-initialization shadow bits,
  `Pool(free: true)` generation use-after-free tags, `bisect` / `sliding`
  monotonicity probe checks, and `law` sampling run in **debug** only (0 cost
  in judge and `--release`).

### `unchecked:` and `cp build --checks`

When a tight kernel (such as an O(n^3) matrix or knapsack loop over a `List` or
`raw` array) retains a dynamic check inside an inner loop, `cp build --checks`
reports the exact line and reason. Wrapping that hot loop in `unchecked:` drops
its remaining bounds checks in judge builds while keeping them active in debug
runs:

```cp
unchecked:
    for i in 0..<n:
        for k in 0..<n:
            let aik = a.raw[i * n + k]
            for j in 0..<n:
                c.raw[i * n + j] += aik * b.raw[k * n + j]
```

---

## 15. Lowering to C++17

Every CP program compiles to a **single, self-contained, readable C++17 file**
(`-std=c++17 -O2`). No external runtime library is required on the judge.

### Compilation Pipeline and the 64 KB Budget

The compiler pipeline runs six stages: parse & desugar -> type/domain check ->
interval/relational range analysis & overflow check -> monomorphize with layout
sharing -> generator push/pull lowering -> tree-shaking & compact C++17
emission.

Four mechanisms guarantee submissions stay well below the **64 KB** judge limit:

1. **Whole-program tree-shaking:** Only reachable functions, methods, and
   stdlib definitions are emitted. Testing hooks (`@brute`, `@checker`,
   `@sample`, `@test`, `@interactor`, `law`, `debug`, `@local`) are stripped
   before emission.
2. **Layout-sharing monomorphization:** All domains (`V`, `E`, `Cell`, `Year`,
   `Cut`) erase to `uint32_t`, so generic containers and algorithms specialized
   on different domains with identical value types share **one** C++ function
   or struct definition.
3. **Minimal runtime preamble:** Fast whole-buffer reader + buffered writer +
   floor-div/mod helpers total ~1.5 KB, and unused parts (e.g., float printer
   or interactive flusher) are omitted when unreferenced.
4. **Embedded `const` budget:** `cp build --size` breaks down emitted bytes per
   module, instantiation, and embedded `const` table; tables over 16 KB warn,
   and exceeding 64 KB total fails the build with an actionable size breakdown.

### Key Lowering Examples (CP -> C++17)

#### 1. Domains, Bounds, Grids, Rings, and `[D]T` Arrays

CP source:

```cp
let (r, c): (1..=1000, 1..=1000) = read()
domain Cell = grid(r, c)
let g: [Cell]Bit = read(digits)
for x in Cell:
    for y in x.adj4:
        if g[y] == g[x]: ...
```

Emitted C++17 (domains erase to `int` / `uint32_t` ordinals, flat vector, zero
bounds checks inside `adj4` because bounds are tested once per direction):

```cpp
int r = rd.read_i32(), c = rd.read_i32();
int Cell_len = r * c;
std::vector<uint8_t> g(Cell_len);
rd.read_digits(g.data(), Cell_len);
for (int x = 0; x < Cell_len; ++x) {
    int xr = x / c, xc = x % c;
    auto body = [&](int y) {
        if (g[y] == g[x]) { /* ... */ }
    };
    if (xr > 0)     body(x - c);
    if (xr + 1 < r) body(x + c);
    if (xc > 0)     body(x - 1);
    if (xc + 1 < c) body(x + 1);
}
```

For `domain V = ring(n)`, `v + k` lowers to `(v + k % n + n) % n` (or simply
`v + 1 == n ? 0 : v + 1` for `v.next`, `v == 0 ? n - 1 : v - 1` for `v.prev`).
For `Cut = I.bounds`, `x.before` lowers to `x`, `x.after` lowers to `x + 1`,
and `b.back(k)` lowers to `b > k ? b - k : 0`.

#### 2. Range Types and Niche Optionals (`?T`)

CP source:

```cp
let a: [V]0..=1e9 = read()
var dist: [V]?Int = none
var par: [V]?V = none
dist[src] = 0
let sum_pair = a[u] + a[v]          // promoted to 64-bit Int
let d = dist[v] ?? -1
```

Emitted C++17 (4-byte storage for `a`, 8-byte niche for `?Int`, 4-byte niche
for `?V`, 64-bit arithmetic):

```cpp
constexpr int64_t INT_NONE = std::numeric_limits<int64_t>::min();
constexpr uint32_t IDX_NONE = std::numeric_limits<uint32_t>::max();

std::vector<uint32_t> a(n);
for (int i = 0; i < n; ++i) a[i] = rd.read_u32();
std::vector<int64_t> dist(n, INT_NONE);
std::vector<uint32_t> par(n, IDX_NONE);
dist[src] = 0;
int64_t sum_pair = int64_t(a[u]) + int64_t(a[v]);
int64_t d = (dist[v] != INT_NONE) ? dist[v] : -1LL;
```

For `dist[v] min= nd`, the compiler emits:
`if (dist[v] == INT_NONE || nd < dist[v]) dist[v] = nd;`.

#### 3. Generators and Sequence Views (Push vs Pull Lowering)

CP source (push lowering — `runs(a)` in a `for` loop):

```cp
var total = 0
for (s, x) in runs(a):
    total += s.len * (s.len + 1) / 2
```

Emitted C++17 (consumer loop body pasted directly at `yield`, zero iterator
objects, zero allocations):

```cpp
int64_t total = 0;
for (int l = 0; l < n; ) {
    int r = l + 1;
    while (r < n && a[r] == a[l]) ++r;
    int64_t slen = r - l;
    total += slen * (slen + 1) / 2;
    l = r;
}
```

When two non-recursive generators are combined with `zip(g1, g2)`, the compiler
emits a small stack struct per generator with an `int state = 0;` switch
machine (`bool next(T& out)`), stepped in lockstep inside a plain `while
(it1.next(x) && it2.next(y))` loop — zero heap allocations, fully inlined.

#### 4. Operation Values and `SegTree` Specialization

CP source:

```cp
var best = SegTree<Cut>(max, update: add)
best.update(top.cuts, a[x] - top.height)
dp[b] = best.max(window)
```

Emitted C++17 (the monoid `max` and lazy action `add` are specialized and
inlined directly into the segment tree struct; no `std::function`, no function
pointers, and `Cut` erases to `int`):

```cpp
struct LazySeg_i64_max_add {
    int n;
    std::vector<int64_t> tr, lz;
    static constexpr int64_t ID = -(std::numeric_limits<int64_t>::max());
    explicit LazySeg_i64_max_add(int n_) : n(n_), tr(4 * n_, 0), lz(4 * n_, 0) {}
    void apply_node(int u, int64_t f) { tr[u] += f; lz[u] += f; }
    void push(int u) {
        if (lz[u] != 0) {
            apply_node(2 * u, lz[u]);
            apply_node(2 * u + 1, lz[u]);
            lz[u] = 0;
        }
    }
    void update(int u, int l, int r, int ql, int qr, int64_t f) {
        if (qr <= l || r <= ql) return;
        if (ql <= l && r <= qr) { apply_node(u, f); return; }
        push(u);
        int m = (l + r) >> 1;
        update(2 * u, l, m, ql, qr, f);
        update(2 * u + 1, m, r, ql, qr, f);
        tr[u] = std::max(tr[2 * u], tr[2 * u + 1]);
    }
    int64_t fold(int u, int l, int r, int ql, int qr) {
        if (qr <= l || r <= ql) return ID;
        if (ql <= l && r <= qr) return tr[u];
        push(u);
        int m = (l + r) >> 1;
        return std::max(fold(2 * u, l, m, ql, qr), fold(2 * u + 1, m, r, ql, qr));
    }
};
```

#### 5. `@multitest` and `static let`

CP source:

```cp
@multitest
static let comb = Comb<Mod<998_244_353>>(upto: 2e5)
let (n, k): (1..=2e5, 0..=n) = read()
if k == 0:
    print(1)
    return
print(comb.binom(n, k))
```

Emitted C++17 (`static let` lifted outside the test loop; `return` simply
returns from `solve_test()`):

```cpp
static const Comb_998244353 comb(200000);

static void solve_test() {
    int n = rd.read_i32(), k = rd.read_i32();
    if (k == 0) {
        wr.writeln(1);
        return;
    }
    wr.writeln(comb.binom(n, k).v);
}

int main() {
    int T = rd.read_i32();
    while (T--) solve_test();
    wr.flush();
}
```

#### 6. `@interactive` and `@grader(init: ...)`

For `@interactive`, `ask("? 1 {r}", as: Int)` lowers to writing the formatted
line, calling `fflush(stdout)`, reading `int64_t res`, and checking
`if (res == -1) std::exit(0);`.

For stateful `@grader(init: fn init(n, x0, y0) -> Int)` (such as Horses), the
compiler lifts all top-level domains, `let`s, and `var`s into a single
`GraderState` struct held in `std::optional<GraderState>`, constructed via
`emplace` inside `init(...)` — guaranteeing zero uninitialized globals and
direct C++ grader ABI compatibility:

```cpp
struct GraderState {
    int n;
    std::vector<uint32_t> x, y;
    std::set<int> big;
    Seg_mod_mul prod;
    Seg_i64_max top;
    int64_t answer() { /* ... */ }
    GraderState(int n_, int x0[], int y0[])
        : n(n_), x(x0, x0 + n_), y(y0, y0 + n_), prod(n_), top(n_) {
        for (int i = 0; i < n; ++i) {
            if (x[i] > 1) big.insert(i);
            prod.set_leaf(i, x[i] % 1000000007);
            top.set_leaf(i, y[i]);
        }
        prod.build(); top.build();
    }
};
static std::optional<GraderState> _st;

int init(int n, int X[], int Y[]) {
    _st.emplace(n, X, Y);
    return (int)_st->answer();
}
int updateX(int pos, int val) {
    auto& s = *_st;
    if (val > 1) s.big.insert(pos); else s.big.erase(pos);
    s.x[pos] = val; s.prod.set(pos, val % 1000000007);
    return (int)s.answer();
}
int updateY(int pos, int val) {
    auto& s = *_st;
    s.y[pos] = val; s.top.set(pos, val);
    return (int)s.answer();
}
```

#### 7. Pools, Handles, `defer`, and `undo`

- `pool Nodes: Node` lowers to `std::vector<Node>` with `uint32_t` indices
  (`UINT32_MAX` for `none`). `nodes.add(v)` lowers to `uint32_t id =
  nodes.size(); nodes.push_back(v); return id;`.
- `defer stmt` is duplicated statically onto every exit edge of its block (no
  closures or dynamic stack).
- `undo dsu, best: body` lowers to saving `auto _m_dsu = dsu.mark(); auto
  _s_best = best;`, running `body`, and emitting `best = _s_best;
  dsu.undo_to(_m_dsu);` on every exit edge of the block.

#### 8. Fast Whole-Buffer I/O

In non-interactive builds, `rd` uses a 64 KB `fread` buffer (`stdin`) and `wr`
uses a 64 KB `fwrite` buffer (`stdout`), parsing integers directly from bytes
in a tight loop (3–5x faster than `std::cin` even with `sync_with_stdio(false)`
while costing ~1.2 KB of emitted C++). In `@interactive` builds, `rd` uses
line/token unbuffered reads and `wr` flushes on every `ask`/`answer`.

---

## 16. Standard Library Specification

Every standard library module is written in CP itself on top of `Buffer<T>`
and compiler intrinsics, tree-shaken to only the functions actually called,
and documented with exact time and space complexity. Prelude items (`Int`,
`List`, `Str`, `read`, `print`, `min`, `max`, `abs`, `sum`, `count`, `any`,
`all`, `bisect`, `prefix_sums`, `runs`, `windows`, `pairs`, `sliding`, `sort`,
`sorted`, `compress`, `grid`, `ring`, `heap_tree`, `Mod`, `SegTree`, `Dsu`,
`Graph`, `Tree`, `bfs`, `Heap`, `SortedSet`, `SortedMap`, `HashMap`, `BitSet`)
are available without explicit `use`; specialized algorithms live in their
named module (`use std.math`, `use std.str`, `use std.geo`, etc.).

Every function below gives its exact signature followed by a single-line cost
annotation (`Cost: time; extra space`). For every standard library container
and data structure, constructing via `Type<...>(args)` is direct shorthand for
`Type.new<...>(args)` (for example `SegTree<Cut>(max, update: add)`,
`Dsu<V>()`, `Heap<(Int, V)>(by: min)`, `Comb<M>(upto: 2e5)`). On any `Iter<T>`,
`.collect()` materializes into the expected collection type (`List<T>`,
`SortedSet<T>`, `HashSet<T>`, etc.) via the `FromIter` trait.

---

### 16.1 `std.seq` — Sequences, Views, Search, and Sorting

#### Prefix Sums and Difference Arrays

`PrefixSums<D, T>` stores an array over `D.bounds` (or doubled cyclic buffer
for `ring(n)`) so range sums never write `pre[r + 1] - pre[l]`:

```cp
// 1D prefix sums over any linear or ring domain (defaults to Group add)
pub fn prefix_sums<D, T>(a: [D]T, op: Group<T> = add) -> PrefixSums<D, T>
// Cost: O(|D|) time; O(|D|) space.

pub fn prefix_sums<D, T>(d: D, f: D -> T, op: Group<T> = add) -> PrefixSums<D, T>
// Cost: O(|D| * cost(f)) time; O(|D|) space.

pub fn prefix_sums<T>(xs: List<T>, op: Group<T> = add) -> PrefixSums<xs.indices, T>
// Cost: O(xs.len) time; O(xs.len) space.

// Query sum over any Range<D> (or cyclic arc on ring(n))
pub fn PrefixSums.sum(self, r: Range<D>) -> T
// Cost: O(1) time; O(1) space.

// 2D prefix sums over a grid or product domain
pub fn prefix_sums_2d<R, C, T>(g: [R, C]T, op: Group<T> = add) -> PrefixSums2D<R, C, T>
// Cost: O(|R| * |C|) time; O(|R| * |C|) space.

pub fn PrefixSums2D.sum(self, rows: Range<R>, cols: Range<C>) -> T
// Cost: O(1) time; O(1) space.

// 1D difference array for offline range additions
pub struct DiffArray<D, T>(...)
pub fn DiffArray.new<D, T>(op: Group<T> = add) -> DiffArray<D, T>
// Cost: O(|D|) time; O(|D|) space.

pub fn DiffArray.add(mut self, r: Range<D>, delta: T)
// Cost: O(1) time; O(1) space.

pub fn DiffArray.finish(self) -> [D]T
// Cost: O(|D|) time; O(|D|) space (reuses backing buffer).
```

#### Binary Search and Partition Points

All binary search functions require a monotone `false -> true` (or `true ->
false` for `last_true`) predicate; debug builds verify monotonicity across
probed points:

```cp
// First value in range where pred holds, or none
pub fn bisect<I>(r: Range<I>, pred: I -> Bool) -> ?I
// Cost: O(log(r.len) * cost(pred)) time; O(1) space.

// Last value in range where pred holds, or none
pub fn last_true<I>(r: Range<I>, pred: I -> Bool) -> ?I
// Cost: O(log(r.len) * cost(pred)) time; O(1) space.

// Cut in D.bounds before the first element where pred holds (for sorted arrays)
pub fn partition_point<D>(r: Range<D>, pred: D -> Bool) -> D.bounds
// Cost: O(log(r.len) * cost(pred)) time; O(1) space.

// Index-space binary search on a range or cyclic arc (returns 0..=r.len)
pub fn Range.bisect_index(self, pred: D -> Bool) -> 0..=self.len
// Cost: O(log(self.len) * cost(pred)) time; O(1) space.

// Float binary search for fixed iterations (avoids infinite eps loops)
pub fn bisect_float(lo: Float, hi: Float, iters: 1..=200 = 80, pred: Float -> Bool) -> Float
// Cost: O(iters * cost(pred)) time; O(1) space.

// Unimodal ternary search on integer range (returns argmin / argmax)
pub fn ternary_min(r: Range1<Int>, f: Int -> Int) -> Int
// Cost: O(log(r.len) * cost(f)) time; O(1) space.
```

#### Sliding Windows and Two Pointers

Two-pointer sweeps exist in two forms: declarative window-predicate helpers
(`longest_window`, `shortest_window`) for O(1)-checkable windows (like prefix
sums), and stateful incremental `sliding` for O(1) add/remove state updates.
`sliding` works uniformly on both linear domains and `ring(n)`!

```cp
// Longest contiguous non-empty window where monotone predicate ok(w) holds
pub fn longest_window<D>(r: Range<D>; ok: Range1<D> -> Bool) -> ?Range1<D>
// Cost: O(r.len * cost(ok)) time; O(1) space.

// Shortest contiguous non-empty window where monotone predicate ok(w) holds
pub fn shortest_window<D>(r: Range<D>; ok: Range1<D> -> Bool) -> ?Range1<D>
// Cost: O(r.len * cost(ok)) time; O(1) space.

// General two-pointer sweep: for each right end (or each start on ring(n)),
// maintains the maximal valid window via incremental add/remove
pub fn sliding<D, S>(
    d: D;
    init: S,
    add: (S, D) -> S,
    remove: (S, D) -> S,
    ok: (Range<D>, S) -> Bool
) yields (Range<D>, S)
// Cost: O(|D| * (cost(add) + cost(remove) + cost(ok))) time; O(sizeof(S)) space.
```

#### Positional and Value Views (All Push-Lowered Generators)

```cp
pub fn pairs<D>(r: Range<D>) yields (D, D)
// Cost: O(r.len) time; O(1) space.

pub fn windows<D>(r: Range<D>, k: 1..) yields Range1<D>
// Cost: O(max(0, r.len - k + 1)) time; O(1) space.

pub fn chunks<D>(r: Range<D>, k: 1..) yields Range1<D>
// Cost: O(ceil_div(r.len, k)) time; O(1) space.

pub fn runs<D, T>(a: [D]T) yields (Range1<D>, T) where T: Eq
// Cost: O(|D|) time; O(1) space.

pub fn runs<D, T, K>(a: [D]T; by: T -> K) yields (Range1<D>, K) where K: Eq
// Cost: O(|D| * cost(by)) time; O(1) space.

pub fn split<D, T>(a: [D]T; at: T -> Bool) yields Range<D>
// Cost: O(|D|) time; O(1) space.

pub fn enumerate<T>(xs: Iter<T>) yields (Int, T)
// Cost: O(|xs|) time; O(1) space.

pub fn zip<A, B>(xs: Iter<A>, ys: Iter<B>) yields (A, B)
// Cost: O(min(|xs|, |ys|)) time; O(1) stack space (pull lowering; non-recursive).

pub fn chain<T>(xs: Iter<T>, ys: Iter<T>) yields T
// Cost: O(|xs| + |ys|) time; O(1) space.

pub fn merge<T>(a: Iter<T>, b: Iter<T>; by: Order<T> = Ord) yields T
// Cost: O(|a| + |b|) time; O(1) space.

pub fn scan<T>(xs: Iter<T>, op: Monoid<T>) yields T
// Cost: O(|xs| * cost(op)) time; O(1) space.
```

#### Sorting, Coordinate Compression, and Permutations

```cp
// In-place sort (pdqsort: O(n log n) worst-case, O(n) on nearly sorted)
pub fn sort<T>(mut xs: List<T>; by: Order<T> = Ord)
pub fn sort<D, T>(mut a: [D]T; by: Order<T> = Ord)
// Cost: O(n log n) time; O(log n) stack space.

// Out-of-place sorted copy (explicitly allocates a new array/list)
pub fn sorted<T>(xs: Iter<T>; by: Order<T> = Ord) -> List<T>
// Cost: O(n log n) time; O(n) space.

// Sort domain positions by key without permuting the backing array
pub fn argsort<D, K>(d: D; by: D -> K) -> List<D> where K: Ord
// Cost: O(|D| log |D|) time; O(|D|) space.

// Coordinate compression: returns fresh domain X, sorted unique vals: [X]T,
// and lookup helper (or mapped [D]X when given a domain array)
pub fn compress<T>(xs: Iter<T>) -> (domain X, vals: [X]T) where T: Ord
// Cost: O(n log n) time; O(n) space.

pub fn compress_array<D, T>(a: [D]T) -> (domain X, vals: [X]T, mapped: [D]X) where T: Ord
// Cost: O(|D| log |D|) time; O(|D|) space.

// Advance in-place to next lexicographic permutation; returns false if wrapped
pub fn next_permutation<D, T>(mut a: [D]T) -> Bool where T: Ord
// Cost: O(|D|) worst-case, O(1) amortized; O(1) space.

// Yield all permutations / k-subsets / submasks lazily
pub fn perms<D>(d: D) yields [D]D
// Cost: O(|D|! * |D|) total; O(|D|) space.

pub fn subsets<D>(d: D) yields List<D>
// Cost: O(2^|D| * |D|) total; O(|D|) space.

pub fn submasks(m: U64) yields U64
// Cost: O(2^popcount(m)) total; O(1) space.

// Longest increasing subsequence length (strict by default)
pub fn lis<D, T>(a: [D]T; strict: Bool = true) -> Int where T: Ord
// Cost: O(|D| log |D|) time; O(|D|) space.
```

#### Reductions

Reductions over a statically non-empty domain or `Range1<D>` return `T` (or
`D` for `argmax`/`argmin`); over a possibly empty range, `List`, or filtered
generator they return `?T` (or `?D`):

```cp
pub fn sum<T>(xs: Iter<T>) -> T where T: Add + Zero
// Cost: O(|xs|) time; O(1) space.

pub fn count<T>(xs: Iter<T>) -> Int
// Cost: O(|xs|) time; O(1) space.

pub fn any(xs: Iter<Bool>) -> Bool
pub fn all(xs: Iter<Bool>) -> Bool
// Cost: O(|xs|) time (short-circuiting); O(1) space.

pub fn min<T>(xs: Iter<T>) -> ?T where T: Ord          // T when provably non-empty
pub fn max<T>(xs: Iter<T>) -> ?T where T: Ord          // T when provably non-empty
pub fn min_by<T, K>(xs: Iter<T>, key: T -> K) -> ?T where K: Ord
pub fn max_by<T, K>(xs: Iter<T>, key: T -> K) -> ?T where K: Ord
pub fn argmin<D, T>(a: [D]T) -> D where T: Ord         // ?D if D may be empty
pub fn argmax<D, T>(a: [D]T) -> D where T: Ord         // ?D if D may be empty
pub fn find<T>(xs: Iter<T>, pred: T -> Bool) -> ?T
pub fn reduce<T>(xs: Iter<T>, op: Monoid<T>) -> T
// Cost (all reductions): O(|xs|) time; O(1) space.
```

Example combining `compress_array`, `runs`, and `bisect`:

```cp
let (X, vals, c) = compress_array(a)
var freq: [X]Int = 0
for i in I: freq[c[i]] += 1
```

---

### 16.2 `std.col` — Collections

Every container fixes its exact representation in its type name — no container
ever switches layout secretly under the hood.

#### `List<T>` and `Deque<T>`

```cp
// List<T>: contiguous growable array (std::vector<T>)
pub fn List.new<T>(cap: Int = 0) -> List<T>           // Cost: O(cap) alloc; O(cap) space.
pub fn List.push(mut self, x: T)                      // Cost: O(1) amortized; O(1) amortized space.
pub fn List.pop(mut self) -> ?T                       // Cost: O(1) time; O(1) space.
pub fn List.last(self) -> ?T                          // Cost: O(1) time; O(1) space.
pub fn List.clear(mut self)                           // Cost: O(1) for Relocate, O(len) otherwise.
pub fn List.reserve(mut self, cap: Int)               // Cost: O(cap) worst-case; O(cap) space.
pub fn List.retain(mut self, pred: T -> Bool)         // Cost: O(len) time; O(1) space.
pub fn List.freeze(self) -> (domain D, [D]T)          // Cost: O(1) time (moves buffer); O(1) extra space.

// Deque<T>: ring-buffer double-ended queue (faster & leaner than std::deque)
pub fn Deque.new<T>(cap: Int = 0) -> Deque<T>         // Cost: O(cap) time; O(cap) space.
pub fn Deque.push_front(mut self, x: T)               // Cost: O(1) amortized.
pub fn Deque.push_back(mut self, x: T)                // Cost: O(1) amortized.
pub fn Deque.pop_front(mut self) -> ?T                // Cost: O(1) time.
pub fn Deque.pop_back(mut self) -> ?T                 // Cost: O(1) time.
pub fn Deque.front(self) -> ?T                        // Cost: O(1) time.
pub fn Deque.back(self) -> ?T                         // Cost: O(1) time.
```

#### `SortedSet<T>`, `SortedMap<K, V>`, and `OrderSet<T>`

`SortedSet<T>` and `SortedMap<K, V>` are balanced BSTs (lowering to
`std::set` / `std::map`). `OrderSet<T>` is an explicit order-statistic tree
(augmented treap / policy-based tree) supporting `kth` and `rank`:

```cp
// SortedSet<T> (ordered BST)
pub fn SortedSet.new<T>(by: Order<T> = Ord) -> SortedSet<T>  // Cost: O(1).
pub fn SortedSet.insert(mut self, x: T) -> Bool              // Cost: O(log n) time; O(1) node.
pub fn SortedSet.remove(mut self, x: T) -> Bool              // Cost: O(log n) time; O(1) space.
pub fn SortedSet.set(mut self, x: T, present: Bool)          // Cost: O(log n) time; O(1) space.
pub fn SortedSet.contains(self, x: T) -> Bool                // Cost: O(log n) time; O(1) space.
pub fn SortedSet.first(self) -> ?T                           // Cost: O(1) time.
pub fn SortedSet.last(self) -> ?T                            // Cost: O(1) time.
pub fn SortedSet.first_ge(self, x: T) -> ?T                  // Cost: O(log n) time (lower_bound).
pub fn SortedSet.first_gt(self, x: T) -> ?T                  // Cost: O(log n) time (upper_bound).
pub fn SortedSet.last_le(self, x: T) -> ?T                   // Cost: O(log n) time.
pub fn SortedSet.last_lt(self, x: T) -> ?T                   // Cost: O(log n) time.
pub fn SortedSet.rev(self) yields T                          // Cost: O(n) total; O(1) space.

// SortedMap<K, V> (ordered BST map)
pub fn SortedMap.new<K, V>(by: Order<K> = Ord) -> SortedMap<K, V> // Cost: O(1).
pub fn SortedMap.get(self, k: K) -> ?V                       // Cost: O(log n) time.
pub fn SortedMap.insert(mut self, k: K, v: V) -> ?V          // Cost: O(log n) time.
pub fn SortedMap.remove(mut self, k: K) -> ?V                // Cost: O(log n) time.
pub fn SortedMap.first_ge(self, k: K) -> ?(K, V)             // Cost: O(log n) time.
pub fn SortedMap.last_le(self, k: K) -> ?(K, V)              // Cost: O(log n) time.

// OrderSet<T> (explicit order-statistic tree: rank & k-th element)
pub fn OrderSet.insert(mut self, x: T) -> Bool               // Cost: O(log n) expected.
pub fn OrderSet.remove(mut self, x: T) -> Bool               // Cost: O(log n) expected.
pub fn OrderSet.kth(self, k: Int) -> ?T                      // Cost: O(log n) expected (0-based).
pub fn OrderSet.count_lt(self, x: T) -> Int                  // Cost: O(log n) expected.
```

#### `HashMap<K, V>` and `HashSet<T>` (Explicit Anti-Hack Hashing)

Open-addressing flat hash table seeded per process with a `splitmix64` random
salt (immune to Codeforces `unordered_map` anti-hash blowups):

```cp
pub fn HashMap.new<K, V>(cap: Int = 0) -> HashMap<K, V> where K: Eq + Hash // Cost: O(cap).
pub fn HashMap.get(self, k: K) -> ?V                         // Cost: O(1) expected.
pub fn HashMap.set(mut self, k: K, v: V)                     // Cost: O(1) amortized expected.
pub fn HashMap.get_or_insert(mut self, k: K, init: () -> V) -> V // Cost: O(1) amortized expected.
pub fn HashMap.remove(mut self, k: K) -> ?V                  // Cost: O(1) expected.
pub fn HashMap.contains(self, k: K) -> Bool                  // Cost: O(1) expected.

pub fn HashSet.new<T>(cap: Int = 0) -> HashSet<T> where T: Eq + Hash // Cost: O(cap).
pub fn HashSet.insert(mut self, x: T) -> Bool                // Cost: O(1) amortized expected.
pub fn HashSet.remove(mut self, x: T) -> Bool                // Cost: O(1) expected.
pub fn HashSet.contains(self, x: T) -> Bool                  // Cost: O(1) expected.
```

#### `Heap<T>` (Binary Heap)

Takes an explicit ordering (`by: min`, `by: max`, or `by: key_fn`) so you never
reverse `<` by mistake:

```cp
pub fn Heap.new<T>(; by: Order<T> = max) -> Heap<T>          // Cost: O(1).
pub fn Heap.from<T>(xs: Iter<T>; by: Order<T> = max) -> Heap<T> // Cost: O(n) Floyd heapify; O(n) space.
pub fn Heap.push(mut self, x: T)                             // Cost: O(log n) time.
pub fn Heap.pop(mut self) -> ?T                              // Cost: O(log n) time.
pub fn Heap.peek(self) -> ?T                                 // Cost: O(1) time.
```

#### `BitSet<D>` and `Mask<D>` (Explicit Packed Sets)

```cp
// BitSet<D>: packed 64-bit word array over domain D (|D|/8 bytes)
pub fn BitSet.new<D>() -> BitSet<D>                          // Cost: O(|D|/64) time; O(|D|/64) words.
pub fn BitSet.set(mut self, i: D, val: Bool = true)          // Cost: O(1) time.
pub fn BitSet.get(self, i: D) -> Bool                        // Cost: O(1) time.
pub fn BitSet.count(self) -> Int                             // Cost: O(|D|/64) time (popcount).
pub fn BitSet.first_set(self) -> ?D                          // Cost: O(|D|/64) time.
pub fn BitSet.next_set(self, after: D) -> ?D                 // Cost: O(|D|/64) worst-case.
pub fn BitSet.shift_left(self, k: 0..) -> BitSet<D>          // Cost: O(|D|/64) time; O(|D|/64) space.
pub fn BitSet.shift_right(self, k: 0..) -> BitSet<D>         // Cost: O(|D|/64) time; O(|D|/64) space.
// Operators &, |, ^, ~, &=, |=, ^= all run in O(|D|/64) word operations.

// Mask<D>: single-word U64 bitmask (requires D.len <= 64, checked on init)
pub fn Mask.empty<D>() -> Mask<D>                            // Cost: O(1) time; 8 bytes.
pub fn Mask.full<D>() -> Mask<D>                             // Cost: O(1) time; 8 bytes.
pub fn Mask.with(self, i: D) -> Mask<D>                      // Cost: O(1) time.
pub fn Mask.without(self, i: D) -> Mask<D>                   // Cost: O(1) time.
pub fn Mask.contains(self, i: D) -> Bool                     // Cost: O(1) time.
pub fn Mask.count(self) -> Int                               // Cost: O(1) time (popcount).
pub fn Mask.iter(self) yields D                              // Cost: O(popcount) time (ctz loop).
```

#### `Pool<T>` and `Journaled<[D]T>`

```cp
// Pool<T> (via `pool Nodes: Node`): flat arena with nominal 4-byte handles
pub fn Pool.new(cap: Int = 0, free: Bool = false) -> Pool<T> // Cost: O(cap) time; O(cap) space.
pub fn Pool.add(mut self, val: T) -> Handle                  // Cost: O(1) amortized.
pub fn Pool.remove(mut self, h: Handle)                      // Cost: O(1) (requires free: true).

// Journaled<[D]T>: rollback-capable array recording old values on writes
pub fn Journaled.new<D, T>(init: T) -> Journaled<[D]T>       // Cost: O(|D|) time; O(|D|) space.
pub fn Journaled.set(mut self, i: D, val: T)                 // Cost: O(1) time; O(1) journal entry.
pub fn Journaled.mark(self) -> Mark                          // Cost: O(1) time.
pub fn Journaled.undo_to(mut self, m: Mark)                  // Cost: O(writes since m) time.
```

---

### 16.3 `std.alg` — Algebra and Range Data Structures

#### Built-in Operation Values

Built-in polymorphic constants: `add` (alias `sum`, `Group`), `mul` (`Monoid`),
`min` (`Idempotent` + `Monoid` when `Bounded`), `max` (`Idempotent` + `Monoid`
when `Bounded`), `xor` (`Group`), `gcd` (`Idempotent` + `Monoid`), `assign`
(`Action`), `affine` (`Action` for `x => a * x + b`), and combinators
`both(m1, m2)` (product monoid), `flip(m)` (reversed order for non-commutative
monoids), and `opt(s)` (lifts a `Semigroup<T>` to a `Monoid<?T>`).

#### `SegTree` (Iterative Point-Update and Lazy Range-Update)

Without `update:`, `SegTree<D, T>(op)` compiles to a tight bottom-up iterative
2n segment tree. With `update: act`, `SegTree<D, T, F>(op, update: act)`
compiles to a lazy-propagation 4n segment tree:

```cp
// Point-update / range-fold segment tree
// `init` accepts either `[D]T` or a zero-copy `D`-indexed view such as `x.map(M)`
pub fn SegTree.new<D, T>(op: Monoid<T>; init: ?[D]T = none) -> SegTree<D, T>
// Cost: O(|D|) build; O(|D|) space (2 * |D| nodes).

pub fn SegTree.set(mut self, i: D, val: T)                   // also `seg[i] = val`
// Cost: O(log |D|) time; O(1) space.

pub fn SegTree.modify(mut self, i: D, f: T -> T)
// Cost: O(log |D|) time; O(1) space.

pub fn SegTree.fold(self, r: Range<D>) -> T                  // aliases .min/.max/.sum when op matches
// Cost: O(log |D|) time; O(1) space.

// Segment-tree walk: largest end cut >= l where pred(fold(l..<end)) holds
pub fn SegTree.max_right(self, l: D, pred: T -> Bool) -> D.bounds
// Cost: O(log |D| * cost(pred)) time; O(1) space.

// Segment-tree walk: smallest start cut <= r.after where pred(fold(start..=r)) holds
pub fn SegTree.min_left(self, r: D, pred: T -> Bool) -> D.bounds
// Cost: O(log |D| * cost(pred)) time; O(1) space.

// Lazy segment tree: range update + range fold + walks
pub fn SegTree.new<D, T, F>(
    op: Monoid<T>;
    update: Action<T, F>,
    init: ?[D]T = none
) -> LazySegTree<D, T, F>
// Cost: O(|D|) build; O(|D|) space (4 * |D| nodes + tags).

pub fn LazySegTree.update(mut self, r: Range<D>, tag: F)
// Cost: O(log |D|) time; O(1) space.

pub fn LazySegTree.fold(mut self, r: Range<D>) -> T
// Cost: O(log |D|) time; O(1) space.
```

#### `Fenwick<D, T>` (Binary Indexed Tree)

Requires `Group<T>` (so passing `max` is a compile error rather than a wrong
answer):

```cp
pub fn Fenwick.new<D, T>(op: Group<T> = add; init: ?[D]T = none) -> Fenwick<D, T>
// Cost: O(|D|) build; O(|D|) space (1 * |D| elements).

pub fn Fenwick.add(mut self, i: D, delta: T)
// Cost: O(log |D|) time; O(1) space.

pub fn Fenwick.prefix(self, b: D.bounds) -> T
// Cost: O(log |D|) time; O(1) space.

pub fn Fenwick.fold(self, r: Range<D>) -> T
// Cost: O(log |D|) time; O(1) space.

// First position whose prefix sum >= target (requires non-negative entries)
pub fn Fenwick.lower_bound(self, target: T) -> D.bounds where T: Ord
// Cost: O(log |D|) time; O(1) space.
```

#### `SparseTable<D, T>` (Static Idempotent RMQ)

Requires `Idempotent<T>` (such as `min`, `max`, `gcd`, bitwise `&`/`|`):

```cp
pub fn SparseTable.new<D, T>(a: [D]T, op: Idempotent<T>) -> SparseTable<D, T>
// Cost: O(|D| log |D|) build; O(|D| log |D|) space.

pub fn SparseTable.fold(self, r: Range1<D>) -> T
// Cost: O(1) time; O(1) space.
```

#### `Dsu<D>` and `RollbackDsu<D>`

```cp
// Standard DSU (path compression + union by size, optional component data monoid)
pub fn Dsu.new<D>() -> Dsu<D>
// Cost: O(|D|) time; O(|D|) space (4 bytes per element).

pub fn Dsu.find(mut self, u: D) -> D
// Cost: O(alpha(|D|)) amortized time; O(1) space.

pub fn Dsu.union(mut self, u: D, v: D) -> Bool               // true if merged two distinct components
// Cost: O(alpha(|D|)) amortized time; O(1) space.

pub fn Dsu.same(mut self, u: D, v: D) -> Bool
// Cost: O(alpha(|D|)) amortized time; O(1) space.

pub fn Dsu.size(mut self, u: D) -> Int
// Cost: O(alpha(|D|)) amortized time; O(1) space.

pub fn Dsu.components(self) -> Int
// Cost: O(1) time; O(1) space.

// RollbackDsu<D> (union by size only, implements Rollback for `undo dsu:`)
pub fn RollbackDsu.new<D>() -> RollbackDsu<D>                // Cost: O(|D|) time; O(|D|) space.
pub fn RollbackDsu.find(self, u: D) -> D                     // Cost: O(log |D|) time (const self!).
pub fn RollbackDsu.union(mut self, u: D, v: D) -> Bool       // Cost: O(log |D|) time; O(1) log entry.
pub fn RollbackDsu.mark(self) -> Mark                        // Cost: O(1) time.
pub fn RollbackDsu.undo_to(mut self, m: Mark)                // Cost: O(merges since m) time.
```

#### `LiChao` and `ConvexHullTrick`

```cp
// LiChao tree over integer x-domain [lo..=hi] for min/max of lines y = m*x + b
pub fn LiChao.new(range: Range1<Int>; by: Order<Int> = min) -> LiChao
// Cost: O(1) init (dynamic pool nodes); O(q log X) space after q insertions.

pub fn LiChao.add_line(mut self, slope: Int, intercept: Int)
// Cost: O(log X) time; O(1) node amortized.

pub fn LiChao.add_segment(mut self, seg: Range1<Int>, slope: Int, intercept: Int)
// Cost: O(log^2 X) time; O(log X) nodes.

pub fn LiChao.query(self, x: Int) -> ?Int
// Cost: O(log X) time; O(1) space.

// Monotone ConvexHullTrick (when slopes are inserted in monotone order)
pub fn ConvexHullTrick.new(; by: Order<Int> = min) -> ConvexHullTrick
// Cost: O(1) time; O(1) space.

pub fn ConvexHullTrick.add_line(mut self, slope: Int, intercept: Int)
// Cost: O(1) amortized time; O(1) space.

pub fn ConvexHullTrick.query(self, x: Int) -> ?Int
// Cost: O(log n) time (or O(1) amortized via query_monotone(mut self, x)); O(1) space.
```

---

### 16.4 `std.graph` — Graphs, Shortest Paths, Connectivity, and Flows

#### Graph Representations (`Graph<V, W>`)

Static graphs use **Compressed Sparse Row (CSR)** storage (two flat arrays,
maximum cache locality, zero per-vertex `std::vector` overhead):

```cp
// Unweighted or weighted CSR graphs constructed from edge iterables
pub fn Graph.undirected<V>(edges: Iter<(V, V)>) -> Graph<V, Unit>
pub fn Graph.directed<V>(edges: Iter<(V, V)>) -> Graph<V, Unit>
pub fn Graph.undirected_w<V, W>(edges: Iter<(V, V, W)>) -> Graph<V, W>
pub fn Graph.directed_w<V, W>(edges: Iter<(V, V, W)>) -> Graph<V, W>
// Cost: O(|V| + |E|) time; O(|V| + |E|) space.

pub fn Graph.adj(self, u: V) yields (to: V, w: W, id: Int)   // for unweighted: yields V
// Cost: O(deg(u)) time; O(1) space.

pub fn Graph.rev(self) -> Graph<V, W>                        // transpose of directed graph
// Cost: O(|V| + |E|) time; O(|V| + |E|) space.

// Mutable adjacency list when edges are added dynamically
pub struct AdjList<V, W = Unit>(...)
pub fn AdjList.add_edge(mut self, u: V, v: V, w: W = ())     // Cost: O(1) amortized.
```

#### Traversals and Shortest Paths

Traversals accept **any** implicit or explicit neighbour generator (`next: u =>
...`), so the exact same `bfs` and `dijkstra` work on `Graph<V>`, `grid(r, c)`
(`x => x.adj4`), and implicit state spaces:

```cp
// Multi-source BFS (unweighted shortest distances)
pub fn bfs<D>(; from: Iter<D>, next: D -> Iter<D>) -> [D]?Int
// Cost: O(|D| + |edges visited|) time; O(|D|) space.

// BFS returning distances and parent pointers for path reconstruction
pub fn bfs_tree<D>(; from: Iter<D>, next: D -> Iter<D>) -> (dist: [D]?Int, parent: [D]?D)
// Cost: O(|D| + |edges visited|) time; O(|D|) space.

// 0-1 BFS (edges with weight 0 or 1, using Deque)
pub fn bfs01<D>(; from: Iter<D>, next: D -> Iter<(D, Bit)>) -> [D]?Int
// Cost: O(|D| + |edges visited|) time; O(|D|) space.

// Dijkstra (non-negative weights, binary heap)
pub fn dijkstra<D>(; from: Iter<D>, next: D -> Iter<(D, 0..)>) -> [D]?Int
// Cost: O((|D| + |E|) log |D|) time; O(|D| + |E|) space.

// Dijkstra with parent reconstruction
pub fn dijkstra_tree<D>(; from: Iter<D>, next: D -> Iter<(D, 0..)>) -> (dist: [D]?Int, parent: [D]?D)
// Cost: O((|D| + |E|) log |D|) time; O(|D| + |E|) space.

// Bellman-Ford (handles negative weights; flags negative cycle reachable from src)
pub fn bellman_ford<V>(edges: Iter<(V, V, Int)>, src: V) -> (dist: [V]?Int, has_neg_cycle: Bool)
// Cost: O(|V| * |E|) time; O(|V|) space.

// Floyd-Warshall all-pairs shortest paths (returns none for unreachable)
pub fn floyd_warshall<V>(init: [V, V]?Int) -> [V, V]?Int
// Cost: O(|V|^3) time; O(|V|^2) space.
```

#### DAGs, Connectivity, SCC, 2-SAT, MST, Eulerian Paths

```cp
// Kahn topological sort; returns none if a cycle exists
pub fn toposort<V>(g: Graph<V, _>) -> ?List<V>
// Cost: O(|V| + |E|) time; O(|V|) space.

// Tarjan strongly connected components (in topological order of condensation DAG)
pub fn scc<V>(g: Graph<V, _>) -> (domain C, comp: [V]C, dag: Graph<C, Unit>)
// Cost: O(|V| + |E|) time; O(|V| + |E|) space.

// 2-SAT solver over boolean variables indexed by domain V
pub struct TwoSat<V>(...)
pub fn TwoSat.new<V>() -> TwoSat<V>                          // Cost: O(|V|) time; O(|V|) space.
pub fn TwoSat.add_or(mut self, u: V, u_val: Bool, v: V, v_val: Bool) // Cost: O(1) amortized.
pub fn TwoSat.add_implies(mut self, u: V, u_val: Bool, v: V, v_val: Bool) // Cost: O(1) amortized.
pub fn TwoSat.solve(self) -> ?[V]Bool                        // Cost: O(|V| + |clauses|) time & space.

// Bridges and 2-edge-connected components (iterative Tarjan, safe on deep graphs)
pub fn bridges<V>(g: Graph<V, _>) -> List<(V, V)>
pub fn two_edge_components<V>(g: Graph<V, _>) -> (domain C, comp: [V]C)
// Cost: O(|V| + |E|) time; O(|V| + |E|) space.

// Articulation points and biconnected components
pub fn articulation_points<V>(g: Graph<V, _>) -> [V]Bool
pub fn biconnected<V>(g: Graph<V, _>) -> List<List<V>>
// Cost: O(|V| + |E|) time; O(|V| + |E|) space.

// Kruskal minimum spanning tree / forest
pub fn mst_kruskal<V, E>(edges: [E](u: V, v: V, w: Int)) -> (total: Int, picked: List<E>)
// Cost: O(|E| log |E|) time; O(|V| + |E|) space.

// Hierholzer Eulerian path / circuit (directed or undirected); none if none exists
pub fn euler_path<V>(g: Graph<V, _>; directed: Bool, start: ?V = none) -> ?List<V>
// Cost: O(|V| + |E|) time; O(|V| + |E|) space.
```

#### Flows and Bipartite Matching

`FlowGraph<V>` manages paired residual edges internally so the classic `e ^ 1`
indexing bug cannot occur:

```cp
pub struct FlowGraph<V>(...)
pub fn FlowGraph.new<V>() -> FlowGraph<V>                    // Cost: O(|V|) time; O(|V|) space.
pub fn FlowGraph.add_edge(mut self, u: V, v: V, cap: 0.., cost: Int = 0) -> FlowEdge
// Cost: O(1) amortized; O(1) space.

// Dinic maximum flow (O(E sqrt(V)) on unit networks, O(V^2 E) worst-case)
pub fn FlowGraph.max_flow(mut self, s: V, t: V) -> Int
// Cost: O(V^2 * E) worst-case (O(E sqrt(V)) unit-cap); O(V + E) space.

// Reachable side of the S-T minimum cut after max_flow
pub fn FlowGraph.min_cut(self, s: V) -> [V]Bool
// Cost: O(V + E) time; O(V) space.

// Successive shortest paths min-cost flow with Johnson potentials (SPFA init if neg costs)
pub fn FlowGraph.min_cost_flow(mut self, s: V, t: V, limit: Int = Int.MAX) -> (flow: Int, cost: Int)
// Cost: O(F * E log V) (or O(V*E + F*E log V) with negative edges); O(V + E) space.

// Hopcroft-Karp maximum bipartite matching between distinct domains L and R
pub fn bipartite_matching<L, R>(edges: Iter<(L, R)>) -> (size: Int, match_l: [L]?R, match_r: [R]?L)
// Cost: O(E * sqrt(|L| + |R|)) time; O(|L| + |R| + E) space.
```

---

### 16.5 `std.tree` — Trees, LCA, Euler Tours, Rerooting, HLD, Centroids

All tree traversals are implemented **iteratively** using explicit stacks, so a
2e5-vertex path graph can never blow the call stack.

```cp
// Unrooted tree constructed from n - 1 edges (checked acyclic+connected in debug)
pub fn Tree.new<V>(edges: Iter<(V, V)>) -> Tree<V>
pub fn Tree.weighted<V, W>(edges: Iter<(V, V, W)>) -> Tree<V, W>
// Cost: O(|V|) time; O(|V|) space.

// Diameter: returns (length, endpoint_a, endpoint_b, path)
pub fn Tree.diameter(self) -> (len: Int, u: V, v: V, path: List<V>)
// Cost: O(|V|) time; O(|V|) space.

// Root a tree at a vertex (iterative BFS/DFS; computes parent, depth, preorder)
pub fn Tree.rooted(self; at: V) -> RootedTree<V, W>
// Cost: O(|V|) time; O(|V|) space.
```

`RootedTree<V, W>` exposes documented ordering guarantees:

- `t.root: V`
- `t.parent: [V]?V` (`none` at `t.root`)
- `t.depth: [V]0..<|V|` (`0` at `t.root`)
- `t.preorder: List<V>` (every parent precedes all its descendants)
- `t.postorder: List<V>` (every child precedes its parent)
- `t.down_edges() yields (parent: V, child: V)` — **guaranteed preorder**:
  when `(p, c)` is yielded, `p` has already been visited.
- `t.up_edges() yields (child: V, parent: V)` — **guaranteed reverse preorder**:
  when `(c, p)` is yielded, all descendants of `c` have already been processed.
- `t.children(v: V) yields V` — `Cost: O(deg(v))`
- `t.subtree_sizes(self) -> [V]1..=|V|` — `Cost: O(|V|) time; O(|V|) space.`

```cp
// LCA and binary lifting (or Euler-RMQ O(1) query via LcaTable)
pub struct Lca<V>(...)
pub fn RootedTree.lca_table(self) -> Lca<V>
// Cost: O(|V| log |V|) build; O(|V| log |V|) space.

pub fn Lca.lca(self, u: V, v: V) -> V                        // Cost: O(1) time (Euler RMQ).
pub fn Lca.dist(self, u: V, v: V) -> Int                     // Cost: O(1) time.
pub fn Lca.kth_ancestor(self, u: V, k: 0..) -> ?V            // Cost: O(log |V|) time (binary lift).
pub fn Lca.jump(self, u: V, v: V, steps: 0..) -> ?V          // Cost: O(log |V|) time.

// Euler tour returning fresh Euler-order domain E, position map, and subtree ranges
pub fn RootedTree.euler_tour(self) -> (domain E, pos: [V]E, subtree: [V]Range1<E>)
// Cost: O(|V|) time; O(|V|) space.

// Generic all-roots rerooting DP (combines child contributions via Monoid)
pub fn RootedTree.reroot<T>(
    self;
    monoid: Monoid<T>,
    add_edge: (T, V, V) -> T,          // lift child DP across edge (parent, child)
    add_root: (T, V) -> T              // finalize vertex DP
) -> [V]T
// Cost: O(|V| * (cost(monoid) + cost(add_edge) + cost(add_root))) time; O(|V|) space.

// Heavy-Light Decomposition returning fresh HLD domain H for SegTree/Fenwick indexing
pub struct Hld<V, H>(...)
pub fn RootedTree.hld(self) -> (domain H, Hld<V, H>)
// Cost: O(|V|) time; O(|V|) space.

pub fn Hld.pos(self, v: V) -> H                              // Cost: O(1) time.
pub fn Hld.subtree(self, v: V) -> Range1<H>                  // Cost: O(1) time.
pub fn Hld.path_ranges(self, u: V, v: V; edge_values: Bool = false) yields (Range1<H>, reversed: Bool)
// Cost: O(log |V|) ranges yielded; O(1) space.

pub fn Hld.lca(self, u: V, v: V) -> V
// Cost: O(log |V|) time; O(1) space.

// Centroid decomposition: invokes callback on each centroid with its alive component
pub fn Tree.centroid_decomp(self, visit: (centroid: V, alive: [V]Bool) -> Unit)
// Cost: O(|V| log |V|) traversal + callback work; O(|V|) space.

// Virtual (auxiliary) tree over a subset of vertices S
pub fn Lca.virtual_tree(self, subset: List<V>) -> (root: V, edges: List<(V, V)>)
// Cost: O(|S| log |S|) time; O(|S|) space.
```

Example (Tree Distances II in 8 lines — zero DFS boilerplate, guaranteed safe
on path graphs):

```cp
let n: 1..=2e5 = read()
domain V = 1..=n
let t = Tree<V>(read(n - 1)).rooted(at: V.first)
let size = t.subtree_sizes()
var ans: [V]Int = 0
ans[t.root] = sum(t.depth[v] for v in V)
for (p, c) in t.down_edges():
    ans[c] = ans[p] + n - 2 * size[c]
print(ans)
```

---

### 16.6 `std.str` — Strings, Pattern Matching, and Automata

`Str` is an owned ASCII byte array exposing a domain `s.indices`, so all
`std.seq` views (`runs`, `windows`, `pairs`, slices `s[l..=r]`) work directly
on strings.

```cp
// String helpers
pub fn Str.positions(self, ch: Char) yields self.indices     // Cost: O(s.len) time; O(1) space.
pub fn Str.split(self, sep: Char) yields Range<self.indices> // Cost: O(s.len) time; O(1) space.
pub fn Str.count(self, ch: Char) -> Int                      // Cost: O(s.len) time; O(1) space.
pub fn Str.starts_with(self, pre: Str) -> Bool               // Cost: O(pre.len) time; O(1) space.
pub fn Str.ends_with(self, suf: Str) -> Bool                 // Cost: O(suf.len) time; O(1) space.
pub fn Str.join<T>(self, xs: Iter<T>) -> Str where T: Show   // Cost: O(total output bytes).

// Z-function: z[i] = LCP of s[0..] and s[i..] (z[0] = s.len)
pub fn z_function<D, T>(a: [D]T) -> [D]0..=|D| where T: Eq
pub fn z_function(s: Str) -> [s.indices]0..=s.len
// Cost: O(n) time; O(n) space.

// KMP prefix function: pi[i] = longest proper border of s[0..=i]
pub fn prefix_function<D, T>(a: [D]T) -> [D]0..=|D| where T: Eq
pub fn prefix_function(s: Str) -> [s.indices]0..=s.len
// Cost: O(n) time; O(n) space.

// KMP pattern search: yields starting positions of pattern in text
pub fn kmp_search(text: Str, pat: Str) yields text.indices
// Cost: O(text.len + pat.len) time; O(pat.len) space.

// Manacher palindromes: odd[i] = max radius centered at char i;
// even[b] = radius centered at gap cut b in s.indices.bounds (no '#' sentinels!)
pub fn manacher(s: Str) -> (odd: [s.indices]Int, even: [s.indices.bounds]Int)
// Cost: O(s.len) time; O(s.len) space.

// Suffix array (SA-IS / doubling) + Kasai LCP + O(1) RMQ LCP helper
pub struct SuffixArray<D>:
    sa: [D]D                            // sorted suffix start positions
    rank: [D]0..<|D|                    // inverse permutation of sa
    lcp: [D]0..<|D|                     // lcp[i] = LCP(sa[i], sa[i + 1])

pub fn suffix_array(s: Str) -> SuffixArray<s.indices>
// Cost: O(n log n) time (O(n) for ASCII SA-IS); O(n) space.

pub fn SuffixArray.lcp_of(self, i: D, j: D) -> Int           // uses internal SparseTable<min>
// Cost: O(1) query after O(n log n) build; O(n log n) space.

// Rolling polynomial hash (random-seeded base mod 2^61 - 1, immune to anti-hash tests)
pub struct RollingHash<D>(...)
pub fn RollingHash.new(s: Str; rng: ?Rng = none) -> RollingHash<s.indices>
// Cost: O(s.len) build; O(s.len) space.

pub fn RollingHash.hash(self, r: Range<D>) -> U64
// Cost: O(1) time; O(1) space.

pub fn RollingHash.concat(h1: U64, h2: U64, len2: Int) -> U64
// Cost: O(1) time; O(1) space.

// Prefix trie backed by a flat Pool (alphabet size explicit, e.g., 26 or 2 for binary trie)
pub struct Trie<const ALPHA: Int>(...)
pub fn Trie.new<const ALPHA: Int>(cap: Int = 0) -> Trie<ALPHA> // Cost: O(cap).
pub fn Trie.insert(mut self, keys: Iter<0..<ALPHA>) -> TrieNode // Cost: O(|keys|) time & nodes.
pub fn Trie.find(self, keys: Iter<0..<ALPHA>) -> ?TrieNode   // Cost: O(|keys|) time; O(1) space.

// Aho-Corasick multi-pattern automaton
pub struct AhoCorasick<const ALPHA: Int>(...)
pub fn AhoCorasick.new<const ALPHA: Int>() -> AhoCorasick<ALPHA> // Cost: O(1).
pub fn AhoCorasick.add(mut self, pat: Iter<0..<ALPHA>) -> Int // Cost: O(|pat|) time & nodes.
pub fn AhoCorasick.build(mut self)                           // Cost: O(total_nodes * ALPHA) time.
pub fn AhoCorasick.step(self, state: AcNode, ch: 0..<ALPHA) -> AcNode // Cost: O(1) time.

// Suffix Automaton (SAM) backed by flat Pool
pub struct SuffixAutomaton<const ALPHA: Int>(...)
pub fn SuffixAutomaton.new<const ALPHA: Int>(cap: Int = 0) -> SuffixAutomaton<ALPHA> // Cost: O(cap).
pub fn SuffixAutomaton.extend(mut self, ch: 0..<ALPHA) -> SamNode // Cost: O(1) amortized; O(1) nodes.
pub fn SuffixAutomaton.distinct_substrings(self) -> Int      // Cost: O(|states|) time; O(1) space.
```

---

### 16.7 `std.math` — Number Theory, Combinatorics, Linear Algebra, and Polynomials

#### Modular Arithmetic and Combinatorics

```cp
// Compile-time static modulus Mod<P> (4 bytes)
pub fn Mod.inv(self) -> Mod<P>                               // requires compile-time prime P
// Cost: O(log P) time; O(1) space.

pub fn Mod.try_inv(self) -> ?Mod<P>                          // works for any P (extended GCD)
// Cost: O(log P) time; O(1) space.

pub fn Mod.pow(self, e: 0..) -> Mod<P>                       // also `a ** e`
// Cost: O(log e) time; O(1) space.

// No-table combinatorics when n is huge (up to 1e18) and k is small (e.g. Bouquet):
pub fn Mod.falling(n: Int, k: 0..) -> Mod<P>                 // n * (n - 1) * ... * (n - k + 1)
// Cost: O(k) time; O(1) space.

pub fn Mod.fact(k: 0..) -> Mod<P>                            // k! mod P directly
// Cost: O(k) time; O(1) space.

// Runtime modulus context (Barrett reduction, zero hidden global state)
pub struct ModRing(mod: 2..=2147483647, ...)
pub fn ModRing.new(m: 2..=2147483647) -> ModRing             // Cost: O(1) time; 16 bytes.
pub fn ModRing.val(self, x: Int) -> DynMod                   // Cost: O(1) time; 4 bytes.
pub fn ModRing.add(self, a: DynMod, b: DynMod) -> DynMod     // Cost: O(1) time.
pub fn ModRing.sub(self, a: DynMod, b: DynMod) -> DynMod     // Cost: O(1) time.
pub fn ModRing.mul(self, a: DynMod, b: DynMod) -> DynMod     // Cost: O(1) time (Barrett).
pub fn ModRing.pow(self, a: DynMod, e: 0..) -> DynMod        // Cost: O(log e) time.
pub fn ModRing.inv(self, a: DynMod) -> ?DynMod               // Cost: O(log m) time.

// Precomputed combinatorics table up to N (use with `static let` in @multitest!)
pub struct Comb<M>(...)
pub fn Comb.new<M>(; upto: 0..=1e7) -> Comb<M>
// Cost: O(upto) time (batch inverse trick); O(upto) space (8 bytes per entry).

pub fn Comb.fact(self, n: Int) -> M                          // Cost: O(1) time.
pub fn Comb.inv_fact(self, n: Int) -> M                      // Cost: O(1) time.
pub fn Comb.binom(self, n: Int, k: Int) -> M                 // returns 0 if k < 0 or k > n; Cost: O(1).
pub fn Comb.perm(self, n: Int, k: Int) -> M                  // Cost: O(1) time.
pub fn Comb.catalan(self, n: Int) -> M                       // Cost: O(1) time.
pub fn Comb.stars_bars(self, items: Int, bins: Int) -> M     // Cost: O(1) time.
```

#### Basic Number Theory, Sieve, and 64-Bit Primality / Factorization

```cp
pub fn gcd(a: Int, b: Int) -> 0..                            // Cost: O(log min(|a|, |b|)) time; O(1) space.
pub fn lcm(a: Int, b: Int) -> 0..                            // Cost: O(log min(|a|, |b|)) time; O(1) space.
pub fn ext_gcd(a: Int, b: Int) -> (g: Int, x: Int, y: Int)   // a*x + b*y == g; Cost: O(log min(|a|, |b|)).
pub fn inv_mod(a: Int, m: 1..) -> ?Int                       // Cost: O(log m) time; O(1) space.
pub fn pow_mod(a: Int, e: 0.., m: 1..) -> 0..<m              // uses I128 mul if m > 2^31; Cost: O(log e).
pub fn crt(r1: Int, m1: 1.., r2: Int, m2: 1..) -> ?(r: Int, m: Int) // Cost: O(log min(m1, m2)).
pub fn floor_div(a: Int, b: Int) -> Int                      // Cost: O(1) time.
pub fn ceil_div(a: Int, b: 1..) -> Int                       // Cost: O(1) time.
pub fn isqrt(n: 0..) -> 0..                                  // exact integer floor(sqrt(n)); Cost: O(1).

// Linear sieve up to limit N
pub struct Sieve(...)
pub fn Sieve.new(upto: 2..=1e7) -> Sieve
// Cost: O(upto) time; O(upto) space (4 bytes per entry for spf).

pub fn Sieve.primes(self) -> List<Int>                       // Cost: O(1) view / ref.
pub fn Sieve.is_prime(self, x: 0..) -> Bool                  // Cost: O(1) time.
pub fn Sieve.spf(self, x: 2..) -> Int                        // smallest prime factor; Cost: O(1).
pub fn Sieve.factorize(self, x: 1..) -> List<(p: Int, e: Int)> // Cost: O(log x) time; O(log x) space.
pub fn Sieve.divisors(self, x: 1..) -> List<Int>             // Cost: O(d(x)) time; O(d(x)) space.
pub fn Sieve.phi_table(self) -> List<Int>                    // Cost: O(upto) time & space.
pub fn Sieve.mobius_table(self) -> List<-1..=1>              // Cost: O(upto) time & space.

// Deterministic 64-bit Miller-Rabin & Pollard's rho (handles up to 1e18)
pub fn is_prime_64(n: 0..) -> Bool
// Cost: O(log^2 n) time (7 deterministic bases); O(1) space.

pub fn factorize_64(n: 1..) -> List<(p: Int, e: Int)>        // sorted prime powers
// Cost: O(n^(1/4) log n) expected time; O(log n) space.
```

#### Polynomials, NTT, Matrices, and Linear Algebra

```cp
// Number-Theoretic Transform convolution (direct for NTT primes like 998_244_353;
// 3-NTT + Garner CRT for arbitrary Mod<P>)
pub fn convolve<M>(a: List<M>, b: List<M>) -> List<M>
// Cost: O(n log n) time where n = a.len + b.len; O(n) space.

pub fn convolve_int(a: List<Int>, b: List<Int>) -> List<Int> // exact via 3-NTT when fits in Int
// Cost: O(n log n) time; O(n) space.

// Polynomial wrapper over Mod<P> (with +, -, *, /, %, derivative, integral, inv, ln, exp)
pub struct Poly<M>(coeffs: List<M>)
pub fn Poly.inv(self, deg: 1..) -> Poly<M>                   // Newton iteration mod x^deg; Cost: O(deg log deg).
pub fn Poly.ln(self, deg: 1..) -> Poly<M>                    // Cost: O(deg log deg) time; O(deg) space.
pub fn Poly.exp(self, deg: 1..) -> Poly<M>                   // Cost: O(deg log deg) time; O(deg) space.
pub fn Poly.pow(self, e: 0.., deg: 1..) -> Poly<M>           // Cost: O(deg log deg) time; O(deg) space.

// Fixed-size and runtime-sized matrices (supports custom Semiring, e.g. min-plus!)
pub struct Matrix<R, C, T>(...)                              // indexed m[r, c]
pub fn Matrix.mul<R, K, C, T>(a: Matrix<R, K, T>, b: Matrix<K, C, T>;
    add: Monoid<T> = add, mul: (T, T) -> T = mul) -> Matrix<R, C, T>
// Cost: O(|R| * |K| * |C|) time; O(|R| * |C|) space.

pub fn Matrix.pow<N, T>(a: Matrix<N, N, T>, e: 0..;
    add: Monoid<T> = add, mul: (T, T) -> T = mul) -> Matrix<N, N, T>
// Cost: O(|N|^3 * log e) time; O(|N|^2) space.

// Gaussian elimination over a Field (Mod<P> or Float)
pub fn gauss_solve<M>(a: [R, C]M, b: [R]M) -> ?(sol: [C]M, null_dim: Int)
// Cost: O(|R| * |C| * min(|R|, |C|)) time; O(|R| * |C|) space.

// Linear XOR basis over GF(2) (up to 64 bits)
pub struct XorBasis(...)
pub fn XorBasis.new() -> XorBasis                            // Cost: O(1) time; 512 bytes.
pub fn XorBasis.insert(mut self, x: U64) -> Bool             // true if linearly independent; Cost: O(64).
pub fn XorBasis.max_xor(self, init: U64 = 0) -> U64          // Cost: O(64) time; O(1) space.
pub fn XorBasis.can_form(self, x: U64) -> Bool               // Cost: O(64) time; O(1) space.
pub fn XorBasis.rank(self) -> 0..=64                         // Cost: O(1) time.
```

---

### 16.8 `std.geo` — 2D Computational Geometry

`Point<T>` is parameterized by its coordinate type (such as `Point<-4e8..=4e8>`
or `Point<Float>`). Keeping integer geometry and floating-point geometry
separate by type ensures exact integer arithmetic never accidentally casts to
`Float`, while coordinate range types let the compiler prove `cross` and `dot`
never overflow:

```cp
pub struct Point<T>(x: T, y: T)
// Supports +, -, unary -, scalar *, ==, lexicographic Ord

pub fn dot<T>(a: Point<T>, b: Point<T>) -> Int               // Float when T == Float
pub fn cross<T>(a: Point<T>, b: Point<T>) -> Int             // Float when T == Float
pub fn norm2<T>(a: Point<T>) -> Int                          // a.x^2 + a.y^2
pub fn dist<T>(a: Point<T>, b: Point<T>) -> Float
// Cost (all basic vector ops): O(1) time; O(1) space.

// Orientation of triple (a, b, c): -1 (CW), 0 (collinear), +1 (CCW)
pub fn orient<T>(a: Point<T>, b: Point<T>, c: Point<T>) -> -1..=1
// Cost: O(1) time; O(1) space.

pub fn on_segment<T>(p: Point<T>, a: Point<T>, b: Point<T>) -> Bool
pub fn segments_intersect<T>(a: Point<T>, b: Point<T>, c: Point<T>, d: Point<T>) -> Bool
// Cost: O(1) time; O(1) space.

// Twice the signed area of a simple polygon (exact integer for integer points)
pub fn polygon_area2<D, T>(poly: [D]Point<T>) -> Int
// Cost: O(|D|) time; O(1) space.

// Point-in-polygon: Inside | Boundary | Outside (O(n) general; O(log n) convex)
enum Pip = Inside | Boundary | Outside
pub fn point_in_polygon<D, T>(poly: [D]Point<T>, p: Point<T>) -> Pip
// Cost: O(|D|) time; O(1) space.

pub fn point_in_convex<D, T>(hull: [D]Point<T>, p: Point<T>) -> Pip
// Cost: O(log |D|) time; O(1) space.

// Andrew's monotone chain convex hull (CCW; optional keep_collinear)
pub fn convex_hull<T>(pts: List<Point<T>>; keep_collinear: Bool = false) -> List<Point<T>>
// Cost: O(n log n) time; O(n) space.

// Exact integer polar-angle sort around origin (half-plane + cross, zero atan2)
pub fn polar_sort<T>(mut pts: List<Point<T>>)
// Cost: O(n log n) time; O(1) extra space.

// Rotating calipers: maximum squared Euclidean distance (diameter) of a convex hull
pub fn hull_diameter2<D, T>(hull: [D]Point<T>) -> (dist2: Int, u: D, v: D)
// Cost: O(|D|) time; O(1) space.

// Divide-and-conquer closest pair of points
pub fn closest_pair<T>(pts: List<Point<T>>) -> (dist2: Int, a: Point<T>, b: Point<T>)
// Cost: O(n log n) time; O(n) space.

// Lines and Circles (Float geometry)
pub struct Line(a: Float, b: Float, c: Float)                // a*x + b*y + c = 0
pub fn Line.intersect(self, other: Line; eps: Float = 1e-9) -> ?Point<Float>
// Cost: O(1) time; O(1) space.

pub struct Circle(center: Point<Float>, r: Float)
pub fn Circle.intersect_line(self, l: Line; eps: Float = 1e-9) -> List<Point<Float>>
pub fn Circle.intersect_circle(self, o: Circle; eps: Float = 1e-9) -> List<Point<Float>>
// Cost: O(1) time; O(1) space.
```

---

### 16.9 `std.io` and `std.interactive` — I/O and Interactive Helpers

```cp
// Standard typed I/O (auto-imported in prelude)
pub fn read<T>() -> T                                        // Cost: O(bytes parsed).
pub fn read<T>(count: Int) -> List<T>                        // Cost: O(count * cost(read<T>)).
pub fn read_digits<D>() -> [D]Bit                            // invoked as `read(digits)`; Cost: O(|D|).
pub fn read_line() -> Str                                    // Cost: O(line length).
pub fn read_until_eof<T>() -> List<T>                        // Cost: O(remaining bytes).
pub fn print(..xs: Show; layout: Layout = space, sep: Str = " ", digits: ?Int = none)
// Cost: O(output bytes).
pub fn flush()                                               // Cost: O(buffered bytes).

// Interactive helpers (active in @interactive files)
pub fn ask<T>(query: Str; as: T) -> T                        // prints, flushes, reads T, exits on fail sentinel
// Cost: O(query.len + reply bytes).

pub fn answer(msg: Str)                                      // prints msg and flushes immediately
// Cost: O(msg.len).
```

For local interactive stress testing without writing shell pipes, `protocol`
and `@interactor` let you run an in-process judge (including adaptive judges)
stripped completely from judge C++ output:

```cp
protocol Guess:
    @wire("?") fn ask(x: 1..=1e9) -> Int
    @wire("!") @final fn answer(x: 1..=1e9)
    limit ask: 40

@interactor(Guess)
struct SecretJudge:
    secret: 1..=1e9
    fn ask(mut self, x: 1..=1e9) -> Int = ...
    fn answer(mut self, x: 1..=1e9) -> Bool = x == self.secret
```

---

### 16.10 `std.rand` — Randomness

All randomness is explicit through an `Rng` value (`xoshiro256**` seeded via
`splitmix64`). There is no hidden global `rand()` state, making randomized
algorithms reproducible in local tests while `Rng.from_clock()` provides
anti-hack entropy on the judge:

```cp
pub struct Rng(...)                                          // 32 bytes, xoshiro256**
pub fn Rng.new(seed: U64 = 0x517cc1b727220a95) -> Rng        // Cost: O(1) time; 32 bytes.
pub fn Rng.from_clock() -> Rng                               // steady_clock + address entropy; Cost: O(1).
pub fn Rng.u64(mut self) -> U64                              // Cost: O(1) time.
pub fn Rng.u32(mut self) -> U32                              // Cost: O(1) time.
pub fn Rng.int(mut self, r: Range1<Int>) -> Int              // unbiased Lemire reduction; Cost: O(1).
pub fn Rng.float(mut self) -> Float                          // uniform in [0, 1); Cost: O(1).
pub fn Rng.bool(mut self, p: Float = 0.5) -> Bool            // Cost: O(1) time.
pub fn Rng.shuffle<D, T>(mut self, mut a: [D]T)              // Fisher-Yates; Cost: O(|D|) time; O(1) space.
pub fn Rng.sample<D>(mut self, d: D) -> D                    // uniform position in non-empty D; Cost: O(1).
pub fn random_hash_seed() -> U64                             // process-unique anti-hack salt; Cost: O(1).
```

---

## 17. Resolution of Open Questions from `00-grounding.md`

The nine grounding solutions in `00-grounding.md` raised four concrete design
questions. Each is resolved below with its rationale and the resulting clean
solution code.

### Open Question 1: Cut Arithmetic (`Tourism`, CCC 2019 S4)

**Question:** Should `b - k` on a bounds domain produce a clipped range
endpoint directly (`b.back(k) ..< b`), or should `b.count` stay the only way
out?

**Resolution:** **Ban raw `x + k` / `x - k` on all linear domains and bounds
domains**, and provide three explicit boundary-policy operations uniformly on
both positions and cuts:

1. Fallible: `x.next`, `x.prev`, `x.offset(k: Int) -> ?D`
2. Explicitly checked: `x.offset(k)! -> D`
3. Clamped (total, never traps): `x.fwd(k: 0..) -> D` and `x.back(k: 0..) -> D`

In addition, `Cut.clip(int_range)` clips an integer range in `0..=n` into a
valid `Range<Cut>`. Keeping `x + k` illegal on linear domains eliminates the
paradox where `Cut.clip(b - k ..< b)` would trap on `b - k` before `clip` even
ran, and directly obeys our guiding test ("could this line compile and trap off
the end?"). Meanwhile `ring(n)` retains `+` and `-` because cyclic arithmetic
is total.

Updated `Tourism` core lines (zero raw `b - k`, zero traps, unified `if`:):

```cp
let (n, k): (1..=1e6, 1..=n) = read()
domain I = 1..=n
let a: [I]1..=1e9 = read()
domain Cut = I.bounds

fn day(b: Cut) -> Int = ceil_div(b.count, k)
fn day_cuts(d: 0..) -> Range<Cut> =
    if d > 0: Cut.clip((d - 1) * k + 1 ..= d * k) else: Cut.first..=Cut.first

var dp: [Cut]Int = 0
var best = SegTree<Cut>(max, update: add)
var stack: List<(height: Int, cuts: Range1<Cut>)> = []

for x in I:
    let b = x.after
    var cuts: Range1<Cut> = x.before ..= x.before
    best[x.before] = dp[x.before] + a[x]
    while let top = stack.last() if top.height <= a[x]:
        best.update(top.cuts, a[x] - top.height)
        cuts = top.cuts.first ..= cuts.last
        stack.pop()
    stack.push((a[x], cuts))
    dp[b] = best.max(day_cuts(day(b) - 1) & (b.back(k) ..< b))

print(dp[Cut.last])
```

### Open Question 2: Grader State Sized by `init` (`Horses`, IOI 2015)

**Question:** How is a domain whose size is set by `init` declared without
uninitialized globals or mutable domain bounds?

**Resolution:** **Unify stateful graders with the file-as-a-function-body rule
via `@grader(init: fn ...)`** (Section 11). In a stateful grader file, the
top-level statements ARE the body of `init`, executing once when the grader
calls `init`, initializing all domains (`domain Year = 0..<n`) and state
(`var x`, `var prod`, `var top`) in strict top-to-bottom order. Every
subsequent `export fn` and `export mut fn` declared in the file accesses that
initialized state cleanly, lowering to a single `std::optional<GraderState>`
constructed via `emplace` inside `init`. This requires zero path-dependent
types, zero uninitialized variables, and zero special `state:` block syntax.

See Section 11 for the full updated `Horses` solution and Section 15 for its
exact C++17 lowering.

### Open Question 3: Specialized Helpers (`dist_sum`, "max dp + range max")

**Question:** Are narrow helpers like `pre.dist_sum(w, to: p[m])` or a "max DP
+ range max" structure stdlib material?

**Resolution:** **No.** We adopt the three-part **Standard Library Inclusion
Criterion** (Section 1): only reusable textbook shapes and named algorithms
with a clear bug-prevention payoff and transparent cost belong in the stdlib.
Problem-specific shortcuts bloat the mental model without improving clarity.

In `Longest Y`, composing our existing primitives (`Range1<P>` subranges and
`.len`) already removes every hand-written `+ 1` / `- 1` while keeping the
median algebra crystal clear:

```cp
fn cost(w: Range1<P>) -> Int:
    let m = w.mid
    let lo = w.first ..< m
    let hi = m ..= w.last
    (p[m] * lo.len - pre.sum(lo)) + (pre.sum(hi) - p[m] * hi.len)
```

### Open Question 4: Two-Pointer vs `bisect` (`One Fourth`, ABC 250 F)

**Question:** Should preferring the clearer O(n log n) `bisect` be the default,
or should O(n) two pointers be equally pleasant?

**Resolution:** **Both must be equally clean, first-class, and explicitly
named** — never swapped silently under the hood. `bisect_index` gives the
declarative O(n log n) solution when that complexity fits; generalizing
`std.seq.sliding` to support cyclic `ring(n)` domains makes the O(n)
two-pointer sweep just as concise and bug-free when tight time limits or
`n = 1e6` demand linear time:

```cp
// O(n log n) with bisect_index (declarative)
for i in V:
    let js = V.arc(from: i + 2, through: i - 1)
    let t = js.bisect_index(j => piece(i, j) >= total)
    for j in js.clipped(t - 1 ..= t):
        best min= abs(total - piece(i, j))

// O(n) with sliding on ring(n) (explicitly linear time, zero manual wrap-around)
for (arc, _) in sliding(V, init: 0,
        add: (s, v) => s + cross(p[v], p[v.next]),
        remove: (s, v) => s - cross(p[v], p[v.next]),
        ok: (arc, s) => 4 * (s + cross(p[arc.last], p[arc.first])) < total):
    best min= abs(total - piece(arc.first, arc.last))
    best min= abs(total - piece(arc.first, arc.last.next))
```

Both algorithms say exactly what they cost, neither has wrap-around or
off-by-one arithmetic, and the programmer chooses freely between them.
