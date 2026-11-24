# btoi17p2

Score: 3/5
Dream: 3/5

Cold review of `port.cp` against `dream.cp`, without the C++. Hops between consecutive blocks are
k x k min-plus tables, a `SegTree` multiplies them, and a query reads one entry. That is the
editorial, but the reads are untyped and the product is hand-written.

## Checklist failures
- idiomatic-code/R5: `let (k, n, m, o): (Int, Int, Int, Int)` (line 12), and the roads and
  orders read as raw `Int`s (lines 39, 41), drop the statement's k <= 5, n <= 5e4, o <= 1e4.
  There is no `// Bounds:` line. Unreported.
- idiomatic-code/R6: places are raw `Int`s split by `/ k` and `% k` at each use (lines 39,
  45-46). A `Place` domain with a `(gap, slot)` view would name them.
- idiomatic-code/R3: `mat_mul` (lines 27-34) is a hand min-plus product; in the friction, with
  repros (`Mat` over `Semiring(min, +)` traps R0401).
- idiomatic-code/R10: `cpl build` 0.05 s against the C++'s 0.03 s (1.67x, the friction's
  measurement) with no performance friction. Filed in the friction now.

## Owed rewrites
- Type the reads from the statement and add `// Bounds:`.
- Stream the orders (`for (a, b) in read<[o](a: Int, b: Int)>()`), as the dream does, instead
  of a `Q` table unpacked by hand on line 44.

## Language gaps
- `Mat` over a non-default semiring, and its size-free identity: in the friction.
- 1.67x at the largest input: added to the friction (perf).

## Dream notes
The dream writes the min-plus product by hand (`fn chain`) to avoid `Mat`'s std::function
fields, where the editorial names a matrix product (caps at 3). It should say
`Mat<Slot, Slot>(over: Semiring(min, +))` and `SegTree(*, hop)` as the friction's ideal does,
and type its reads.
