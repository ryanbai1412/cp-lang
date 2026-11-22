# Delta: cp-lang 0.2.2 -> 0.3 (the ideal spec) [first cut, 2026-11-21]

Method: every ```cp block in `spec/` and `sketches/exemplars-16-25.md` (56 blocks) plus 25 one-feature probes, run
through `cpl check` at cp-lang `3869a20`. 1 of 56 spec blocks checks today; most stop at the first parse error, so
this list is a floor. Probes and results: `out/probe/`.

## A. Already matches (no work)
`domain V = 0..n`, range types, `?T` with absorbing `+`, `??` with diverging right side, `let … else`, postfix `?`,
user traits and `impl`, operator impls (`impl Add for P`), `@memo`, `Mod<p>` with a runtime `p`, `for … if` filter,
`mut` at declaration and call site, `use std.x.{Name}`, floor `/` `%`, `Int` niche value.

## B. Syntax: parse-level additions (mechanical)
| 0.3 | status | note |
|---|---|---|
| `repeat n:` | missing | sugar for `for _ in 0..n` |
| `loop:` | missing | |
| `enum Dir: N, E, S, W` one-line enums; `labels(…)` | missing | |
| one-line `struct Case: a: T, b: U` | missing | |
| `fn f(…) -> T: ...` stubs | missing | spec-only; skip |
| `for x in s where c` | 0.2.2 spells it `for x in s if c` | pick one; spec says `where` |
| `(l, +1)` unary plus | missing | |
| `var a = 0, b = 1` multi-binding | missing | |
| `Str(len: lo..=hi)` refined string type | missing | |
| struct type-param defaults `Graph<V, W = ()>` | missing | |

## C. Core semantics (real compiler work)
| 0.3 | status |
|---|---|
| `domain V = read(1..=n)` and domains from any expression | front end parses; back ends refuse (E0501) |
| existential returns `-> (domain P, …)`, `let (domain P, x) = …` | missing |
| escape rules 12.2 and per-declaration slots 12.3 with recursive save/restore | to audit in codegen |
| `[(A, B)]T ≡ [A][B]T` product arrays | 0.2.2 has `[A, B]T` instead; decide spelling |
| `Self` in traits, associated types/consts, `Index` as a user-implementable trait | missing |
| trait-based `?T` lifting (`impl<T: Add> Add for ?T`) | lifting is built in |
| `@derive(...)` | parsed, back ends refuse; needs the comptime reflection decision |
| library decorators/markers (`@droppable`, user decorators) | refused |
| `.Cut` on arrays / domains, `x.before`/`x.after` | 0.2.2 has `V.cuts`, `v.l`/`v.r`; reconcile |
| `x.Cut`-style anonymous-array domains | missing |
| range inference model A (`let` keeps, `var` widens) | to audit |

## D. Stdlib surface changes
| 0.3 | 0.2.2 |
|---|---|
| `yes(b)`; `Bool` has no `Print` | `print(true)` prints `YES` |
| `Wire` for interaction | `ask(…)`, `@interactor` |
| `adjacency`, `Tree(…).rooted(at:)`, `components` returning a fresh domain | `Graph.undirected`, `Rooted(…)` |
| `binom`, `Z.factorials` | `Comb<P>()` |
| `Bits<D>` | `Subset<D>` |
| `Map<K, V> = 0` default value | missing |
| `SegTree<D, M, A>(init:)`, `Monoid` as a trait | `SegTree<D,T>(+, a)`, `Monoid<T>(id:, op:)` value |
| `Mint(mod: p)` returning a fresh type | `Mod<P>` |

## E. Deferred (keep 0.2.2 behaviour)
Program shape (`--- with multitest() ---` vs `@!multitest`), settings (`with io(…)` / `#set` vs `read(none: 0)` and
friends), test roles (`@alt`/`@gen`/`@check` vs `@brute`/`@gen`/`@checker`).

## F. Codegen / runtime
Big-stack thread from the judge profile (none today); 8-byte `?Int` everywhere it's free (today only proven
tables); runtime-sized arrays never on the stack (audit).

## Open decisions this surfaces for Ryan
1. `for … where` vs keep `for … if`.
2. `[(A, B)]T` vs keep `[A, B]T` for 2-D arrays.
3. Derives: comptime field/variant reflection vs macros (blocks `@derive`, `labels`, interaction messages).
4. `Bits<D>` vs `Subset<D>`; `Mint(mod:)` vs `Mod<P>` (library naming; I'd decide unless you care).
