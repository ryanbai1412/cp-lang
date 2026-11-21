# 14. Functions, traits, decorators

**Layer:** core C5, C8, C12. Status: [proposed].

## 14.1 Functions and lambdas

**What.** `fn` declarations (top-level or nested) and `params => e` lambdas.

**Why nested functions are the chunking mechanism.** Long solutions split into named steps that share the
test's state without threading it through parameters [you]. A nested function captures its enclosing
activation by reference, so splitting a block into chunks is a cut-and-paste, not a redesign.

**Layer.** Core.

- `fn f(x: T) -> U = expr` or a block body. Functions **hoist** within their block; every capture must be
  initialized before the first call (checked, chapter 21 diagnostics style).
- Nested functions capture by reference to the enclosing activation. This is the default way to split a
  long solution into chunks: each chunk is a nested function with its inputs as parameters. A nested function
  can have a nested `@alt` sibling sharing its captures, compared at real call sites; it moves to the top
  level only to be stressed alone with its own `@gen` [you, r2; p3 proposed].
- Function values (lambdas, nested functions) can't leave their declaring block or go into a static or
  field, by the same rule as domains, so capture by reference is safe [r11 N13].
- `params => e` is the only lambda. Parameters are patterns, so `(u, v) => e` takes two arguments or one
  pair, whichever is expected. `fn` parameters may be patterns too, but stay typed:
  `fn paths((r1, c1): Pt, (r2, c2): Pt)` [r16 R3]. There are no separate argument labels. Arrays and bound methods are functions.
- Named arguments: options are always named, never bare flags. A bare variant name in a named argument
  resolves against that parameter's enum (`order: desc`). Keywords may be argument labels (`first(where:)`).

## 14.2 Closures and purity

**What.** A function type is `(A, B) -> R` or `pure (A, B) -> R`. A `pure` function can't do I/O, mutate
captures, or call non-`pure` functions; calls to `@memo` functions count as pure [r16 R2].

**Why core.** Lazy adapters (`filter`, `map` on views) and `@memo` need purity in their *signatures*, so it
must be part of the type, not a rule about which stdlib function you're calling [r15].

- **Closures.** Nested functions and lambdas capture enclosing `var`s by reference and may mutate them, except
  lambdas passed to lazy adapters (22.5). A function-typed parameter is written `(A, B) -> R`, and a nullary
  lambda `() => e`. Function parameters don't escape; a call with a statically known function value is
  specialized per value, never through a pointer. `cp explain` lists the clones with their bytes.

**Cost.** Calls with a statically known function value are specialized per value; no indirect calls.

**Rejected.** Effect systems with polymorphism (too much); "pure" as a lint (lazy views would be unsound).

## 14.3 Traits, generics and operators

**What.** `trait` declares methods, associated types and laws; `impl Trait for T` implements them. Generics
take trait bounds. Operators are trait methods: `a + b` is `Add.add(a, b)`, `a[i]` is `Index`/`At`,
`a max= b` is `a = max(a, b)` over `Ord`.

**Why.** It's what lets operator lifting over `?T` (`23`), `Mint` (`24`) and `Max`/`Min` monoids (`27`) be
library code, and it's how user types plug into `SegTree` and `read`.

**Rules.**
- **Definition-time checking** [you]: a generic body is checked against its bounds once, not per
  instantiation, so errors point at the generic or the call, never inside a stdlib template.
- **Library-authored diagnostics** [you]: a trait may declare `#on_unimplemented("…")` and per-law messages,
  so `SegTree<V, M>` with a non-monoid `M` says what to write.
- **Return-type dispatch:** a call can be resolved by its expected type (`let x: Int = read()`). This is all
  `read<T>()` is; no I/O special case.
- Coherence: one impl per (trait, type), declared with the trait or the type. Blanket impls are allowed
  (`impl<T: Add> Add for ?T`).
- **Inherent impls** (`impl T:` with no trait) are legal only in the module that declares `T`. A blanket
  inherent impl `impl<A: Tr> A:` is legal only in the module that declares `Tr`; its methods resolve as if
  they were `Tr`'s, which is how `21`'s position methods attach to every `Index` [2impl Q56].
- **Method resolution** [2impl Q58]: `x.m(…)` looks for `m` first among inherent methods of `x`'s type, then
  among methods of traits in scope that the type implements. Two candidates at the same step are an
  ambiguity error naming both; `Tr.m(x, …)` picks one. Auto-deref and auto-ref don't exist.
- **No overloading by type** [2impl Q63]: a function is identified by its name plus its parameter labels, so
  `s.first()` and `s.first(after: x)` are different functions, and `sorted()` / `sorted(by:)` likewise.
  Two declarations with the same name and labels in one scope are an error. Type-directed variants
  (`Graph.undirected` on weighted or unweighted edges) are one generic over a trait.

**Cost.** Monomorphized; no vtables.

**Rejected.** Templates checked at instantiation (C++ errors); type classes with orphan instances.

## 14.4 Decorators and markers

**What.** `@d(args)` before a declaration. Two kinds, one syntax:
- **Wrapping decorators:** `@d fn f(…)` means `f` is `d(f_body)`, where recursion inside `f_body` calls the
  decorated `f`. A decorator is an ordinary generic function over function types.
- **Role markers:** the declaration is tagged for a tool (`@alt`, `@gen`, `@check`, `@interactor`; chapter
  30) and is otherwise untouched. A marker is declared by its library with `marker`.

Markers before the first statement of a file attach to the **program**, which is a declaration. A
decorator or marker never affects code it isn't attached to.

**Layer.** The syntax and the two meanings are core. Every decorator and marker except `@grader` and `@cpp`
is library [p4].

**Rejected.** `@` for configuration (that's `with`, chapter 17); macros (no diagnostics or editor support).

### `@memo` (library decorator)

`@memo` is `fn memo<A: Index, R>(f: pure (A) -> R) -> (A) -> R` with the table `[A]?R`; a multi-parameter
function's key is the tuple of its parameters, which is `Index` (chapter 13). `@memo(hash)` uses a
`HashMap`. "A `@memo` function can't do I/O" is the `pure` bound, not a special rule.

```cp
@memo
fn count(i: 0..=k.len, r: 0..d, tight: Bool) -> Z: ...
```

```cp
@memo
fn count(i: 0..=k.len, r: 0..d, tight: Bool) -> Z: ...
```

- **Table choice is explicit** [r7, r10 33]. When every parameter has a bounded type, the cache is a dense
  array over the product of those bounds (`[0..=k.len][0..d][Bool]?Z`); `cp explain` shows its bytes, and
  over the profile's memory it is a compile-time warning. Otherwise it is a compile error naming
  `@memo(hash)`. A dense table never silently becomes a hash map.
- **Lifetime:** the cache lives as long as the block declaring the function: per test for a top-level function
  (16.3), per activation inside another function (`upto(k)` gets a fresh cache per call).
- A `@memo` function can't do I/O, mutate captured state, or capture anything but `let` bindings of
  values that can't change while the cache lives. A write to a captured array after the `@memo` declaration
  in the same block is a compile error; writes through other paths hit the debug generation check (16.1)
  [r11 N14, r12 R11].
  It can't outlive the values its bounds mention.
- `cp explain` shows each table's bytes and fill cost per activation [r11 N19].
- Mutual recursion between `@memo` functions is allowed; each has its own table.
- Domains, cuts and bounded ints all count as bounded [p2 S5]. `@memo(hash)` is open addressing over the key's
  storage width. Lowering may run leading non-recursive guards before the probe (unobservable, since `@memo`
  is pure); the dense table is still sized by the types [r14 T18].
- **`@memo` tables.**
  - A dense table is allocated on the first call.
  - A parameter bound may mention earlier parameters (`j: 0..=i`). The table is then jagged: row offsets are
    computed once, and `cp explain` shows the total.
  - The cache entry is `?R` even when `R` is optional, stored with a second niche or a flag byte.

### `multitest` (library function)

Multitest is the library function `multitest(count:, prefix:, body:)`, used as `--- with multitest() ---`; see 16.3.

### `@droppable` (library marker)

Marks a function whose result may be dropped without the unused-result lint. `push`, `pop`, `mark`,
`union` and `insert` use it; so can any library [p4].
