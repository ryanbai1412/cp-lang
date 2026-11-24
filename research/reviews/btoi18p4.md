# btoi18p4 review

Score: 3/5
Dream: 3/5

Cold review of `port.cp` against `dream.cp`, without the C++. The header derives the nesting
argument and the cases: even or one outermost wire, three, or two neighbours sharing a
direction. The port is that case split. Wires are unrolled to `lo..=hi` with `hi` past `n`
when they wrap, and arcs are swept by hand. The friction reports that.

## Checklist failures
- idiomatic-code/R6: the tops are a `List<Wire>` read back through `tops[Int(t)]` (lines 68,
  82, 97, 99), next to `domain T` and `rank: [Wire]?T`. A `[T]Wire` table would keep the
  domain.
- idiomatic-code/R3: the ring arithmetic (`s in [-n, 0, n]` laps, `covers_ring`'s `far`, and
  `+ (if ...: n else: 0)` at lines 20, 98) is hand-written; in the friction (no ring range
  type).

## Owed rewrites
- Make `tops` a `[T]Wire` (`T.wrap` already gives the neighbours).
- Add a `cpl run` for the friction's ring-range claim (idiomatic-code/R11).

## Language gaps
- A ring/cyclic range type with nesting and coverage: in the friction, without a run.

## Dream notes
The friction says dream.cp has "a ring domain with `RingRange` reads, `Seg.ring.nesting(arc)`
... `coverage(arcs)`". The dream.cp on disk has none of that: it is the port with a different
holder sweep and `group(by:)`. It keeps the same `tops[Int(t)]` and unrolled arcs, and reads
`n: 1..=1e5` where the port has `3..=1e5` (caps at 3). Write the ring-domain dream the friction
describes (send back).
