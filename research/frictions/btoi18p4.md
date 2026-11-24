# btoi18p4 frictions

dream.cp has the ideal: a ring domain with `RingRange` reads, `Seg.ring.nesting(arc)` for the
outermost wires and their nested ones, and `Seg.ring.coverage(arcs).min_over(part)`.

## Ring arcs by hand

std has no ring/cyclic range type, so each wire is unrolled to `lo..=hi` with `hi` past `n` when
it wraps, and `covers` repeats each arc one lap either way before a sorted sweep. The nesting
forest comes from a sweep over the doubled circle sorted by `(start, desc(end))`, and the tops
get their own domain `T` with `T.wrap` for neighbours round the ring.
