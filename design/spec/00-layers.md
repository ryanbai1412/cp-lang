# Spec 0: what is core, what is library, what is sugar [proposed]

Every feature in the spec belongs to exactly one layer:

| layer | meaning | test |
|---|---|---|
| **core** | the compiler must know it: typing rules, layout, control flow, lowering | can't be written in the language |
| **library** | ordinary code in the language, possibly calling a listed intrinsic | a user library could ship the same thing |
| **sugar** | syntax with a written desugaring into core + library | the desugaring is one rule, shown |

Goal: keep core small (target ≈ 15 mechanisms), so "is this special?" has a short answer, and the stdlib has
no powers a user library lacks.

## 1. The core surface

| # | mechanism | why it can't be library |
|---|---|---|
| C1 | syntax, layout (indent blocks), literals, precedence | the parser |
| C2 | value types: `Int` (with its niche), fixed widths, `F64`, `Byte`, tuples, structs, payload enums, function types | layout |
| C3 | **range-refined integers** (`lo..=hi`) with interval arithmetic and contract checks | typing rule over every operator |
| C4 | **`?T`**: niche layout, postfix `?` propagation, `if let`, `??` (lazy right side) | control flow + layout |
| C5 | traits and generics, checked at definition; **operators are trait calls**; return-type-directed dispatch (`read<T>()`) | type system |
| C6 | **scoped (fresh) types**: `let (domain V, …) = e`, existential returns, the escape check, hidden runtime parameters (sizes, moduli) | the one genuinely new type-system idea |
| C7 | **arrays `[D]T` over the `Index` trait**; dense row-major layout; `[(A, B)]T ≡ [A][B]T` | layout + type equality |
| C8 | functions, nested functions (hoisting), non-escaping closures, **pure function types** (for lazy adapters and `@memo`) | effects are a typing rule |
| C9 | control flow: `if`, `for … where`, `while`, `loop`, labels, `repeat`, exhaustive `match`, `Never` | the IR |
| C10 | value semantics: copy on assignment, moves of temporaries, debug invalidation checks | memory model |
| C11 | **contexts**: `context`, `with`, `uses` (lexical, compile-time) | implicit compile-time arguments |
| C12 | **declarations with roles**: `@name(args)` = apply a library-declared decorator or tool role | attaches to declarations |
| C13 | checks and traps: bounds, ranges, overflow; debug traps, release wraps or traps per rule | semantics of both builds |
| C15 | the program: the file body, wrap lines `--- with f() ---`, entry points (16.3); lowering contract to C++17 (zero cost, bundling, 64 KB) | the output |

**Intrinsics** (core functions the library calls, each one line of C++): byte reader/writer, allocation,
checked and wrapping arithmetic, `mulhi`/128-bit multiply, `popcount`/`ctz`/`clz`, `trap`, stderr, timer,
seeded RNG for `gen`, and `@cpp` as the escape hatch.

## 2. Index spaces, by layer

| layer | pieces |
|---|---|
| core | C6 (fresh scoped types, escape check, hidden sizes), C7 (`[D]T` over `Index`, product equality) |
| library | the `Index` trait; counted domains, `labels`, `Cut`, `ring`, `grid`, `Bool`, `Letter`; every position method (`fwd`, `next`, `dist`, `written`); `numbered`, `compress`, `components`, `bfs` returning fresh domains |
| sugar | `domain V = e` ≡ `let (domain V, _) = Domain.of(e)`; `labels(a, b)` ≡ an enum deriving `Index`; `xs.Cut` on an anonymous array |

```cp
trait Index:                                   // core knows this trait: it's what [D]T requires
    fn count() -> Int
    fn ord(self) -> 0..Self.count
    fn from_ord(i: 0..Self.count) -> Self

fn Domain.of(n: 1..) -> (domain D: Index)       // the stdlib's counted domain: ordinary code over C6
```

So `[V]Int`, `[Cut]Int`, `[Letter]Int` and `[(Row, Col)]Int` are one core rule; everything that makes them
pleasant is library.

## 3. I/O, by layer

| layer | pieces |
|---|---|
| core | the byte intrinsics; return-type dispatch (C5), which is all `read<T>()` is; contexts (C11) |
| library | `Read`/`Print` traits and every impl; `context io { base, sep, answers, precision }`; base conversion = the `Read` impl for counted positions, which `uses io`; `yes()`; `Wire`; `[V]List<V>` and `Graph` reads |
| sugar | string interpolation `"? {v}"` ≡ `Print` calls into the buffer |

```cp
impl<D: Counted> Read for D uses io:
    fn read() -> D = D.at(read<Int>() - io.base)      // that's the whole of 1-based input
```

`print(true)` failing is also library: `Bool` doesn't implement `Print`, and the trait's library-authored
diagnostic names `yes(b)`.

## 4. Everything else, by layer

| feature | layer | note |
|---|---|---|
| operator lifting over `?T` | **library** | blanket `impl<T: Add> Add for ?T`; "ordering doesn't lift" = no `Ord` impl |
| `max=` / `min=` | sugar | `x = Max.op(x, y)`; `?T` skipping is the `Max` monoid's identity |
| `if x min= y:` | core (C9) | the one assignment-as-condition rule |
| `for x in s where c` | sugar | `for x in s: if not c: continue` |
| `??`, multi-binding `if let` | core (C4) | laziness and `Never` |
| sequences, views, folds, scans, windows, `pairs` | library | purity of lazy-adapter lambdas is C8 in their signatures |
| `Mint`, `Z.factorials` | library | over C6 (runtime modulus) + `mulhi` |
| containers, `Heap(order:)`, `Dsu`, `Graph`, `bfs` | library | |
| `SegTree`, `Fenwick`, `Monoid`/`Action` | library | |
| `@memo` | **library** (decorator) | table = `[(A, B, …)]?R` because a tuple of `Index` is `Index`; `@memo(hash)` a `HashMap`; purity from C8 |
| `multitest` | **library** (a function; `--- with multitest() ---` is sugar, 16.3) | calls the per-test closure; bindings above the line are per program |
| `@grader`, `@cpp` | core (C15) | ABI and the escape hatch |
| `@alt`, `@gen`, `@check`, `@interactor`, `@mock` | tool roles | declared by the stress library, consumed by `cp stress`, stripped |
| `gen<T>()` | library | a `Gen` trait with derived impls; the shrinker records the RNG intrinsic |
| `with checks(…)`, `with lint(…)` | library contexts read by core (C13) and the linter | |
| lints | library | each lint is declared by the library that owns the names it mentions |

## 5. Magic this exposes (things special today that shouldn't be)

1. **Flush rule.** Today the compiler decides by whether `Wire` is reachable. Library version: constructing a
   `Wire` switches stdout to interactive at runtime (one branch per flush point). No reachability analysis.
2. **Operator lifting** is written as a language rule; it's three blanket impls.
3. **`@memo` and multitest** are compiler features; `@memo` is a C12 decorator and `multitest` an ordinary function used through a wrap line (16.3).
4. **"Stdlib functions marked droppable"**: make it `@droppable`, usable by any library.
5. **Depth inference** was a growing list of six shapes. Deleted: the program runs on a big stack and
   nothing tracks depth (18.3).
6. **Lints that know stdlib names** (`len`/`count` in the range lint): move to the library owning the names.
7. **`Bool` and `Str` special cases**: `Bool` is `labels(false, true)` with literals; `Str` indexing by `Int` is
   its `Index` impl.

Each item is a [proposed] change, separate from this classification.

## 6. Proposed file layout

| part | files |
|---|---|
| 0 | `00-layers.md` (this), `01-principles.md` (planned: one page from ALIGN/GOALS) |
| core | `10-syntax`, `11-types` (C2–C4), `12-scoped-types` (C6), `13-arrays` (C7), `14-functions-traits` (C5, C8, C12), `15-control` (C9), `16-state` (C10, statics), `17-contexts` (C11), `18-checks-lowering` (C13–C15) |
| library | `20-intrinsics`, `21-index-spaces`, `22-sequences`, `23-optionals`, `24-modular`, `25-io`, `26-containers`, `27-algebra-segtrees`, `28-graphs-trees`, `29-strings` |
| tools | `30-stress`, `31-interaction`, `32-graders-bundling`, `33-debug-trace` |
| appendix | exemplars, idioms, decision log, open questions |

Every entry uses one template: **what** · **why** (the bug or friction it removes) · **layer** (desugaring or
library code) · **cost** (lowering) · **tooling** (how `cp explain`, the tracer and the LSP show it) ·
**rejected alternatives**.
