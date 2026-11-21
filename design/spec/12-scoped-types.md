# 12. Scoped (fresh) types

**Layer:** core C6. Status: [proposed].

This is the only new type-system mechanism in the language. Domains (`21`), runtime moduli (`24`) and
anything else sized or parameterized by an input value are library code built on it.

## 12.1 Fresh types

**What.** A binding can introduce a new type whose identity is that binding:

```cp
domain V = read(1..=1e5)                      // sugar, see 12.4
let (domain Piece, pieces) = t.components(without: c)
type Z = Mint(mod: p)
```

Two fresh types are never equal, even when built from the same expression: `domain A = n; domain B = n`
gives `[A]T ≠ [B]T`.

**Why.** "Vertex vs edge", "row vs column" and "a residue mod p vs mod q" are distinctions only the program
knows. Without fresh types they collapse to `Int` and the type checker can't help.

**Layer.** Core: creating the type, its identity, and the escape check (12.2). What the type *does* comes
from ordinary trait impls on the constructor (`Domain.of`, `Mint`), which are library.

**Rejected.** Generative types via functors (ML), too heavy; branded lifetimes (Rust's `GhostCell` trick),
unreadable; dependent types, far too much.

## 12.2 Scope and escape

**What.** A fresh type is block-scoped and declared before use. A value whose type mentions it can't
leave the block. "Leaving" is exactly this list [you, 2026-11-21]:

| path | verdict |
|---|---|
| `return` (or a tail expression) out of the declaring block | error, unless the function's return type declares it existentially (below) |
| assignment into a `var` declared outside the block | error |
| a closure that captures such a value and is stored or returned past the block | error; a closure passed down to a call is fine |
| a per-program binding (16.3) | error, so a position can't carry over to the next test |
| a struct field whose type mentions it | allowed only in a struct declared inside the block |
| an argument to a call | always fine: the block is still active for the call's duration |

**Why.** It makes "an index from test 3 used in test 4" and "a table built under one modulus read under
another" compile errors [you, r10 16].

**Rules.**
- A generic with statics or `@memo` can't be instantiated at a fresh type.
- Tool roots (`@interactor`, `@gen`, `@check`) live at program level and can't capture one [r6 4]. `@alt`
  can, because it's substituted at its call site.
- A function may *return* a fresh type existentially: `fn components(…) -> (domain P, [V]P)`. The caller
  must bind it with the keyword (`let (domain P, …) = …`); a capitalized name alone is rejected [r2].
  User code can write such functions; the stdlib has no special power here.
- Tuple components bind left to right, so later types may mention earlier names:
  `let (domain A, k): (1..=1e6, 1..=A.count) = read()` [r9 20].

**Diagnostics.** The escape error names the type, the block, and the escaping path.

## 12.3 Hidden runtime parameters

**What.** A fresh type carries runtime data: a domain its size, `Mint(mod: p)` its modulus. Each
declaration site owns one static slot holding that data [you, 2026-11-21: CORE-ALIGN design 4A].

- Executing the declaration writes the slot. Generic code instantiated at the type reads the slot directly;
  it takes no hidden parameters, and closures and structs over the type store nothing extra.
- Generic code is monomorphized per declaration site, so two fresh types never share a slot.
- When the declaring block can be re-entered while active (a recursive function declaring a domain, e.g.
  one per centroid-decomposition level), entry saves the slot and exit restores it, on every exit path
  including `return` and traps caught by tooling.

**Why it's sound.** By 12.2, a value whose type mentions a fresh type exists only while its declaring
block is active, and the innermost active activation is the one whose data is in the slot. A recursive
callee can't receive the outer activation's values: its parameter types can't name a domain declared in
its own body.

**Why.** So `for x in D`, `[D]Bool` and `Z(3) * Z(5)` work inside generic code without passing `n` or `p`
by hand, and recursion stays correct.

**Cost.** One store per declaration, one load where generic code uses it, and a save/restore per
re-entrant activation (omitted when the block can't be re-entered). A compile-time constant
(`Mint(mod: 998244353)`) is specialized away. `cp explain` says which. Debug checks that each `[D]T`
argument has length `D.count` [r4 18, r5 12, r10 14–15].

**Rejected.** Hidden arguments (dictionary passing): more rules (closures, struct fields) and real extra
parameters. Explicit witnesses (`bfs(V, …)`): plumbing at every call. Static slots with re-entry
forbidden: rules out per-level domains in recursion.

**Rules.**
- `V.count` is typed by the declared range, so `let n = V.count` keeps its bound [r10 30]. An existential
  domain inherits its source's maximum [r10 28].
- A domain parameter that no value parameter mentions is passed explicitly: `f<Col, Row>(r)` [p1 S3].
- Going from a constant parameter to a runtime one is one local edit.

## 12.4 Sugar

| written | means |
|---|---|
| `domain V = e` | `let (domain V, _) = Domain.of(e)` |
| `domain V = read(1..=n)` | `domain V = read<1..=n>()` |
| `type Z = Mint(mod: p)` | `let (type Z, _) = Mint.of(p)` |

Sizes never convert under `io(base:)`; only positions do [r9 16].
