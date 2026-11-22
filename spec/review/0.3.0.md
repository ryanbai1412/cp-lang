# Independent review of the 0.3.0 draft: resolutions

One reviewer read the whole tree at 0f945b5c and filed 40 findings (T, C,
A, G, R codes below). Each line says what was decided and where it landed.
Findings without a resolution are still open.

| # | Finding | Resolution |
|---|---|---|
| T1 | "release never has UB" is false for unchecked indexes and `x!` | Promise narrowed to Ryan's rule: release adds no UB to valid programs and avoids it where free; an invalid program gets what its C++ does. 00, 09.2, 13.1 |
| C1 | `Slice` stub kept 0-based List/Str slices | Stub rewritten to "every slice keeps its source's keys". std/seq.cp |
| A1 | postfix `?` both local and early return | `?` is local propagation only; early exit is `let x = e else: return none`. 02, 05 §6 and §8 |
| A2 | slices passed where `List<T>` / `[V]T` expected | `[K]T` parameters are view types (any `Indexed`/`IndexedMut`, walk `a.keys`); `List<T>` and `Str` parameters take only that container. 04b, 07 |
| C2 | `?T` ordered or not | Not `Ord` (chapter 2 wins); ops-ranges line fixed |
| A3 | `io(base: 1)` on `1..=n` | `io.base` applies only to counted domains (`[n]`); explicit-end domains read unshifted. 10.2; overview program fixed |
| A4 | literal at a range end: position or cut | A literal end is a position unless the other end is a cut. 04 |
| C3 | constructors move vs lend | Generated constructors move (they store); custom constructors are static fns and lend. 02.6, 05 §4 |
| A5 | `let ys = xs` / `var ys = xs` on a lent parameter | `let` re-lends; `var` of a lent heap value is an error suggesting `.clone()`. 05 |
| C9 | `@get` parameters | `@get` may take defaulted parameters. 02 |
| C10 | check names differ | One list in 09.3: bounds, ranges, overflow, unwrap, div, asserts, reads |
