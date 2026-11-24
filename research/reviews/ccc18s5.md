# ccc18s5

Score: 3/5
Dream: 3/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the product-graph Kruskal.
- R2: two `Dsu`s and each kind weighted by the other side's component count read like the editorial.
- R2/R4: links are packed into one `Int` as `cost << 18 | k` (L23) and unpacked with `>> 18` and `% (1 << 18)` (L30): a C++ sort trick the report now describes; the enum overflow that forced it is fixed upstream.
- R5/R6: `Planet`, `City`, `F`, `G` domains and ranged reads.
- R10: 0.03 s vs 0.04 s.

## Owed rewrites

- Drop the bit packing: walk `merge_sorted` (std) of the two cost-sorted link lists, or sort a list of `(cost:, link:)` records and `match` the link.

## Language gaps

None: std's `merge_sorted` is the merged walk (`cpl run` in the report).

## Dream notes

`sum(merge(...).map(e => match e: ... cities.unite(...) ...))` performs the unions inside the sum (R7), which caps the dream at 3; it should be a `for` loop over the merged links. Provisional until rewritten.
